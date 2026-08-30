use hecs::World;
use std::io::Write;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

/// Number of real-world seconds between invocations of the registered tick
/// functions in `App::run`. Defaults to once per second.
pub const TICK_PER_SECONDS: u64 = 1;

/// Shared handle to the ECS world. Used both by the main loop (which will
/// eventually run the Update schedule) and by the HTTP server thread.
pub type SharedWorld = Arc<Mutex<World>>;

/// Top-level application state.
pub struct App {
    pub world: SharedWorld,
    pub startups: Vec<Box<dyn FnMut(SharedWorld)>>,
    pub ticks: Vec<Box<dyn FnMut(SharedWorld)>>,
}

impl App {
    /// Create a new app with a fresh hecs world and no startup or tick functions.
    pub fn new() -> Self {
        App {
            world: Arc::new(Mutex::new(World::new())),
            startups: Vec::new(),
            ticks: Vec::new(),
        }
    }

    /// Register a startup function. It will be invoked once, just before the
    /// main loop starts. The function receives a `SharedWorld` handle and can
    /// lock it (e.g. to spawn entities) or clone it (e.g. to move into a
    /// spawned thread).
    pub fn register_startup(&mut self, f: impl FnMut(SharedWorld) + 'static) {
        self.startups.push(Box::new(f));
    }

    /// Register a tick function. It will be invoked once per `TICK_PER_SECONDS`
    /// from `run`'s main loop, after the startup functions have run.
    pub fn register_tick(&mut self, f: impl FnMut(SharedWorld) + 'static) {
        self.ticks.push(Box::new(f));
    }

    /// Clone the shared world handle. Pass this to subsystems (e.g. tests or
    /// future features) that need to read or mutate entities from another
    /// thread. Currently unused by the main binary but kept as part of the
    /// public API.
    #[allow(dead_code)]
    pub fn world_handle(&self) -> SharedWorld {
        Arc::clone(&self.world)
    }

    /// Run the main loop, blocking until Ctrl+C (SIGINT) or SIGTERM.
    /// First executes every registered startup function, then enters a loop
    /// that waits `TICK_PER_SECONDS` for either a signal or a timeout, and
    /// on each timeout runs every registered tick function.
    ///
    /// The loop body is intentionally minimal — it is reserved for the
    /// future Update schedule. The HTTP server runs on its own thread.
    pub fn run(&mut self) {
        // Run all startup functions before entering the loop. Each one gets
        // its own clone of the world handle so it can move the Arc into
        // another thread if it needs to.
        for mut startup in self.startups.drain(..) {
            startup(Arc::clone(&self.world));
        }

        // Use a Condvar so the main thread wakes immediately when the signal
        // fires, instead of having to wait out a tick.
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
            // Sleep until either Ctrl+C fires (notify_all wakes us) or the
            // tick duration elapses (timeout wakes us).
            let (guard, _) = cvar.wait_timeout(running, tick_duration).unwrap();
            running = guard;

            // If the wakeup was from Ctrl+C, skip the ticks and exit.
            if !*running {
                break;
            }

            // Otherwise, fire every registered tick function. Each one gets
            // its own clone of the world handle.
            for tick in self.ticks.iter_mut() {
                tick(Arc::clone(&self.world));
            }
        }

        println!("Shutting down gracefully.");
        std::io::stdout().flush().unwrap();
    }
}
