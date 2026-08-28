use hecs::World;
use std::io::Write;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

/// Top-level application state.
pub struct App {
    pub world: World,
    pub startups: Vec<Box<dyn FnMut(&mut World)>>,
}

impl App {
    /// Create a new app with a fresh hecs world and no startup functions.
    pub fn New() -> Self {
        App {
            world: World::new(),
            startups: Vec::new(),
        }
    }

    /// Register a startup function. It will be invoked once, just before the
    /// main loop starts. The function receives `&mut World` and can spawn
    /// entities (or do any other one-shot setup).
    pub fn RegisterStartup(&mut self, f: impl FnMut(&mut World) + 'static) {
        self.startups.push(Box::new(f));
    }

    /// Run the main loop, blocking until Ctrl+C (SIGINT) or SIGTERM.
    /// First executes every registered startup function.
    pub fn Run(&mut self) {
        // Run all startup functions before entering the loop.
        for mut startup in self.startups.drain(..) {
            startup(&mut self.world);
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
