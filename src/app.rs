use hecs::World;
use std::io::Write;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

/// Top-level application state.
pub struct App {
    pub world: World,
}

impl App {
    /// Create a new app with a fresh hecs world.
    pub fn New() -> Self {
        App {
            world: World::new(),
        }
    }

    /// Run the main loop, blocking until Ctrl+C (SIGINT) or SIGTERM.
    /// Uses a Condvar so the main thread wakes immediately when the signal fires,
    /// instead of having to wait out a long sleep.
    pub fn Run(&mut self) {
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
