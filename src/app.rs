use hecs::World;
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

/// Number of real-world seconds between invocations of the registered tick
/// functions in `App::run`. Defaults to once per second.
pub const TICK_PER_SECONDS: u64 = 1;

/// Shared handle to the ECS world. Used both by the main loop (which will
/// eventually run the Update schedule) and by the HTTP server thread.
pub type SharedWorld = Arc<Mutex<World>>;

/// Shared pause flag. Set by the HTTP layer, read by `App::run` each tick.
pub type SharedPaused = Arc<AtomicBool>;

/// Top-level application state.
pub struct App {
    pub world: SharedWorld,
    pub is_tick_paused: SharedPaused,
    pub startups: Vec<Box<dyn FnMut(SharedWorld)>>,
    pub ticks: Vec<Box<dyn FnMut(SharedWorld)>>,
}

impl App {
    /// Create a new app with a fresh hecs world and no startup or tick functions.
    pub fn new() -> Self {
        App {
            world: Arc::new(Mutex::new(World::new())),
            is_tick_paused: Arc::new(AtomicBool::new(false)),
            startups: Vec::new(),
            ticks: Vec::new(),
        }
    }

    /// Register a startup function invoked once before the main loop.
    /// The function receives a `SharedWorld` handle it can lock or move into a thread.
    pub fn register_startup(&mut self, f: impl FnMut(SharedWorld) + 'static) {
        self.startups.push(Box::new(f));
    }

    /// Register a tick function. It will be invoked once per `TICK_PER_SECONDS`
    /// from `run`'s main loop, after the startup functions have run.
    pub fn register_tick(&mut self, f: impl FnMut(SharedWorld) + 'static) {
        self.ticks.push(Box::new(f));
    }

    /// Clone the shared world handle for subsystems that need it from
    /// another thread. Currently unused but kept as part of the public API.
    #[allow(dead_code)]
    pub fn world_handle(&self) -> SharedWorld {
        Arc::clone(&self.world)
    }

    /// Clone the pause flag for subsystems that toggle it, such as the
    /// HTTP server thread.
    pub fn pause_handle(&self) -> SharedPaused {
        Arc::clone(&self.is_tick_paused)
    }

    /// Run the main loop until SIGINT/SIGTERM, running all ticks every
    /// `TICK_PER_SECONDS`. The loop body is reserved for the future Update schedule.
    pub fn run(&mut self) {
        // Run all startup functions before entering the loop, cloning
        // the world handle per call so each can move the Arc into a thread.
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

            // Skip the ticks while paused. The loop keeps waiting on the
            // condvar so Ctrl+C still wakes us immediately.
            if self.is_tick_paused.load(Ordering::Relaxed) {
                continue;
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
