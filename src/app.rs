use hecs::World;
use std::collections::HashMap;
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

/// Real-world seconds between auto-firings of the `on_tick` schedule.
pub const TICK_PER_SECONDS: u64 = 1;

/// Shared handle to the ECS world. Used by the main loop, HTTP handlers,
/// and any thread that needs to read or mutate entities.
pub type SharedWorld = Arc<Mutex<World>>;

/// Shared handle to the app state, cloned into every startup and handler.
pub type SharedApp = Arc<AppState>;

/// One boxed handler behind its own Mutex so the outer map lock can be
/// released before a handler runs and so re-entrant fires do not deadlock
/// on a single handler instance.
pub type ScheduleHandler =
    Arc<Mutex<Box<dyn FnMut(SharedApp, SharedWorld, SharedSchedules) + Send + 'static>>>;

/// Maps schedule name → ordered list of handlers. Newtype over `Arc<Mutex>`
/// so inherent methods are allowed (orphan rule) and so it is cheap to
/// clone into every handler as `SharedSchedules`.
pub struct SharedSchedules(Arc<Mutex<SchedulesMap>>);

/// Backing map for the schedule registry.
#[derive(Default)]
pub struct SchedulesMap {
    map: HashMap<String, Vec<ScheduleHandler>>,
}

impl Clone for SharedSchedules {
    fn clone(&self) -> Self {
        SharedSchedules(Arc::clone(&self.0))
    }
}

impl SharedSchedules {
    /// Wrap a fresh empty registry as a clonable handle.
    pub fn new() -> Self {
        SharedSchedules(Arc::new(Mutex::new(SchedulesMap::default())))
    }

    /// Register a handler under `name`. Multiple handlers per name run in
    /// registration order each time the schedule fires.
    pub fn register(
        &self,
        name: &str,
        f: impl FnMut(SharedApp, SharedWorld, SharedSchedules) + Send + 'static,
    ) {
        let handler: ScheduleHandler = Arc::new(Mutex::new(Box::new(f)));
        self.0
            .lock()
            .expect("schedules mutex poisoned")
            .map
            .entry(name.to_string())
            .or_default()
            .push(handler);
    }

    /// Run every handler under `name` in order. Re-entrant; no-op if none.
    pub fn fire(&self, name: &str, app: SharedApp, world: SharedWorld) {
        let handlers = match self
            .0
            .lock()
            .expect("schedules mutex poisoned")
            .map
            .get(name)
        {
            Some(v) => v.clone(),
            None => return,
        };
        for handler in handlers {
            let mut h = handler.lock().expect("schedule handler mutex poisoned");
            h(app.clone(), world.clone(), self.clone());
        }
    }
}

/// State shared by startups and handlers. Interior mutability so every
/// holder can read and write through an `&self`.
pub struct AppState {
    is_tick_paused: AtomicBool,
}

impl AppState {
    fn new() -> Self {
        AppState { is_tick_paused: AtomicBool::new(false) }
    }

    /// Pause or resume the tick loop. Callable from any thread.
    pub fn set_pause(&self, is_paused: bool) {
        self.is_tick_paused.store(is_paused, Ordering::Relaxed);
    }

    /// Whether the tick loop is currently paused.
    pub fn is_paused(&self) -> bool {
        self.is_tick_paused.load(Ordering::Relaxed)
    }
}

/// Top-level application state.
pub struct App {
    pub state: SharedApp,
    pub world: SharedWorld,
    pub schedules: SharedSchedules,
    pub startups: Vec<Box<dyn FnMut(SharedApp, SharedWorld)>>,
}

impl App {
    /// Create a new app with a fresh world and empty schedule registry.
    pub fn new() -> Self {
        App {
            state: Arc::new(AppState::new()),
            world: Arc::new(Mutex::new(World::new())),
            schedules: SharedSchedules::new(),
            startups: Vec::new(),
        }
    }

    /// Pause or resume the tick loop from the owning thread.
    #[allow(dead_code)]
    pub fn set_pause(&self, is_paused: bool) {
        self.state.set_pause(is_paused);
    }

    /// Register a startup function invoked once before the main loop.
    pub fn register_startup(&mut self, f: impl FnMut(SharedApp, SharedWorld) + 'static) {
        self.startups.push(Box::new(f));
    }

    /// Register a handler under `name`. Multiple handlers per name run in
    /// registration order each time the schedule fires.
    pub fn register_schedule(
        &mut self,
        name: &str,
        f: impl FnMut(SharedApp, SharedWorld, SharedSchedules) + Send + 'static,
    ) {
        self.schedules.register(name, f);
    }

    /// Fire all handlers registered under `name` on the calling thread.
    #[allow(dead_code)]
    pub fn fire(&self, name: &str) {
        self.schedules
            .fire(name, Arc::clone(&self.state), Arc::clone(&self.world));
    }

    /// Clone the shared world handle for subsystems that need it from
    /// another thread.
    #[allow(dead_code)]
    pub fn world_handle(&self) -> SharedWorld {
        Arc::clone(&self.world)
    }

    /// Clone the shared app-state handle for subsystems that need it.
    #[allow(dead_code)]
    pub fn app_handle(&self) -> SharedApp {
        Arc::clone(&self.state)
    }

    /// Run the main loop until SIGINT/SIGTERM, auto-firing the `on_tick`
    /// schedule every `TICK_PER_SECONDS`.
    pub fn run(&mut self) {
        // Run all startup functions before entering the loop, cloning the
        // handles per call so each can move the Arcs into a thread.
        for mut startup in self.startups.drain(..) {
            startup(Arc::clone(&self.state), Arc::clone(&self.world));
        }

        // Condvar so the main thread wakes immediately on Ctrl+C instead
        // of waiting out the rest of a tick.
        let pair = Arc::new((Mutex::new(true), Condvar::new()));
        {
            let pair = Arc::clone(&pair);
            ctrlc::set_handler(move || {
                let (lock, cvar) = &*pair;
                let mut running = lock.lock().unwrap();
                *running = false;
                cvar.notify_all();
            })
            .expect("failed to set Ctrl+C handler");
        }

        println!("\nRunning. Press Ctrl+C to stop.");
        std::io::stdout().flush().unwrap();

        let tick_duration = Duration::from_secs(TICK_PER_SECONDS);
        let (lock, cvar) = &*pair;
        let mut running = lock.lock().unwrap();
        while *running {
            // Sleep until Ctrl+C (notify_all) or the tick elapses.
            let (guard, _) = cvar.wait_timeout(running, tick_duration).unwrap();
            running = guard;

            if !*running {
                break;
            }

            // Skip the schedule while paused. The condvar still wakes us
            // immediately on Ctrl+C.
            if self.state.is_paused() {
                continue;
            }

            // Auto-fire the on_tick schedule every TICK_PER_SECONDS.
            self.schedules
                .fire("on_tick", Arc::clone(&self.state), Arc::clone(&self.world));
        }

        println!("Shutting down gracefully.");
        std::io::stdout().flush().unwrap();
    }
}