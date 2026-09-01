use hecs::World;
use std::collections::HashMap;
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Real-world seconds between auto-firings of the `on_tick` schedule.
pub const TICK_PER_SECONDS: u64 = 1;

/// Shared handle to the ECS world. Used by the main loop, Tauri commands,
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
    /// Optional Tauri handle so game handlers can emit events to the webview.
    /// `None` when running headless without a Tauri window.
    tauri: Mutex<Option<tauri::AppHandle>>,
}

impl AppState {
    fn new() -> Self {
        AppState {
            is_tick_paused: AtomicBool::new(false),
            tauri: Mutex::new(None),
        }
    }

    /// Pause or resume the tick loop. Callable from any thread.
    pub fn set_pause(&self, is_paused: bool) {
        self.is_tick_paused.store(is_paused, Ordering::Relaxed);
    }

    /// Whether the tick loop is currently paused.
    pub fn is_paused(&self) -> bool {
        self.is_tick_paused.load(Ordering::Relaxed)
    }

    /// Attach the Tauri handle so handlers can `emit` events.
    pub fn set_tauri(&self, handle: tauri::AppHandle) {
        *self.tauri.lock().expect("tauri mutex poisoned") = Some(handle);
    }

    /// Borrow the Tauri handle, if attached. Returns `None` in headless mode.
    pub fn tauri(&self) -> Option<tauri::AppHandle> {
        self.tauri.lock().expect("tauri mutex poisoned").clone()
    }
}

/// Top-level application state.
pub struct App {
    pub state: SharedApp,
    pub world: SharedWorld,
    pub schedules: SharedSchedules,
    pub startups: Vec<Box<dyn FnMut(SharedApp, SharedWorld) + Send>>,
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

    /// Register a startup function invoked once before the main loop.
    pub fn register_startup(&mut self, f: impl FnMut(SharedApp, SharedWorld) + Send + 'static) {
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

    /// Drain and run every registered startup, cloning shared handles per call.
    pub fn run_startups(&mut self) {
        for mut startup in self.startups.drain(..) {
            startup(Arc::clone(&self.state), Arc::clone(&self.world));
        }
    }

    /// Run the tick loop on the calling thread until `stop` is cleared.
    /// Honours `AppState::is_paused` by skipping the schedule while paused.
    /// Shutdown is bounded by `TICK_PER_SECONDS` since `park_timeout` returns
    /// at the next tick and re-checks the flag.
    pub fn run_tick_loop(
        app: SharedApp,
        world: SharedWorld,
        schedules: SharedSchedules,
        stop: Arc<AtomicBool>,
    ) {
        println!("\nRunning. Close the window to stop.");
        std::io::stdout().flush().unwrap();

        let tick_duration = Duration::from_secs(TICK_PER_SECONDS);
        while stop.load(Ordering::Relaxed) {
            std::thread::park_timeout(tick_duration);
            if !stop.load(Ordering::Relaxed) {
                break;
            }
            if app.is_paused() {
                continue;
            }
            schedules.fire("on_tick", app.clone(), world.clone());
        }
        println!("Shutting down gracefully.");
        std::io::stdout().flush().unwrap();
    }
}
