use hecs::World;
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

/// Real-world seconds between auto-firings of the `on_tick` schedule.
pub const TICK_PER_SECONDS: u64 = 1;

/// Shared handle to the ECS world. Cloned into every command and handler.
pub type SharedWorld = Arc<Mutex<World>>;

/// Shared handle to the app state, cloned into every startup and handler.
pub type SharedApp = Arc<AppState>;

/// State shared by handlers. Interior mutability so every holder can
/// read and write through an `&self`.
pub struct AppState {
    pub is_tick_paused: AtomicBool,
    /// Attached in `setup`; read by tick handlers to emit events to the webview.
    tauri: OnceLock<tauri::AppHandle>,
}

impl AppState {
    fn new() -> Self {
        AppState {
            is_tick_paused: AtomicBool::new(false),
            tauri: OnceLock::new(),
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

    /// Attach the Tauri handle so handlers can `emit` events. Idempotent.
    pub fn set_tauri(&self, handle: tauri::AppHandle) {
        self.tauri.set(handle).expect("tauri handle already set");
    }

    /// Borrow the Tauri handle, if attached. `None` before `setup` runs.
    pub fn tauri(&self) -> Option<&tauri::AppHandle> {
        self.tauri.get()
    }
}

/// Top-level application state.
pub struct App {
    pub state: SharedApp,
    pub world: SharedWorld,
}

impl App {
    /// Create a new app with a fresh world and empty app state.
    pub fn new() -> Self {
        App {
            state: Arc::new(AppState::new()),
            world: Arc::new(Mutex::new(World::new())),
        }
    }

    /// Run the tick loop on the calling thread until `stop` is cleared.
    /// Honours `AppState::is_paused` by skipping the schedule while paused.
    pub fn run_tick_loop(
        app: SharedApp,
        world: SharedWorld,
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
            crate::game::date_advancing::tick(app.clone(), world.clone());
        }
        println!("Shutting down gracefully.");
        std::io::stdout().flush().unwrap();
    }
}
