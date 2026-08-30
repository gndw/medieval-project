use hecs::World;
use std::io::Write;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

/// Shared handle to the ECS world. Used both by the main loop (which will
/// eventually run the Update schedule) and by the HTTP server thread.
pub type SharedWorld = Arc<Mutex<World>>;

/// Top-level application state.
pub struct App {
    pub world: SharedWorld,
    pub startups: Vec<Box<dyn FnMut(SharedWorld)>>,
}

impl App {
    /// Create a new app with a fresh hecs world and no startup functions.
    pub fn new() -> Self {
        App {
            world: Arc::new(Mutex::new(World::new())),
            startups: Vec::new(),
        }
    }

    /// Register a startup function. It will be invoked once, just before the
    /// main loop starts. The function receives a `SharedWorld` handle and can
    /// lock it (e.g. to spawn entities) or clone it (e.g. to move into a
    /// spawned thread).
    pub fn register_startup(&mut self, f: impl FnMut(SharedWorld) + 'static) {
        self.startups.push(Box::new(f));
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
    /// First executes every registered startup function.
    ///
    /// The loop body is intentionally empty for now — it is reserved for the
    /// future Update schedule. The HTTP server runs on its own thread.
    pub fn run(&mut self) {
        // Run all startup functions before entering the loop. Each one gets
        // its own clone of the world handle so it can move the Arc into
        // another thread if it needs to.
        for mut startup in self.startups.drain(..) {
            startup(Arc::clone(&self.world));
        }

        // Use a Condvar so the main thread wakes immediately when the signal
        // fires, instead of having to wait out a long sleep.
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

        let (lock, cvar) = &*pair;
        let mut running = lock.lock().unwrap();
        while *running {
            let (guard, _) = cvar.wait_timeout(running, Duration::from_secs(60)).unwrap();
            running = guard;
        }

        println!("Shutting down gracefully.");
        std::io::stdout().flush().unwrap();
    }
}
