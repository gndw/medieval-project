mod app;
mod commands;
mod components;
mod content;
mod game;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use app::App;
use tauri::Manager;

fn main() {
    let app = App::new();

    // Shared stop flag for the tick loop; flipped when the window closes.
    let tick_stop = Arc::new(AtomicBool::new(true));
    let tick_stop_for_window = Arc::clone(&tick_stop);

    tauri::Builder::default()
        .setup(move |tauri_app| {
            // Attach the Tauri handle so game handlers can emit events.
            app.state.set_tauri(tauri_app.handle().clone());

            // Register shared state so commands can extract it via `State<T>`.
            tauri_app.manage(app.state.clone());
            tauri_app.manage(app.world.clone());

            // Load content into the world.
            content::startup(app.state.clone(), app.world.clone());

            // Spawn the tick loop on its own OS thread.
            let stop = Arc::clone(&tick_stop);
            std::thread::Builder::new()
                .name("medieval-tick".into())
                .spawn(move || {
                    App::run_tick_loop(app.state.clone(), app.world.clone(), stop);
                })
                .expect("failed to spawn tick thread");
            Ok(())
        })
        .on_window_event(move |_window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                tick_stop_for_window.store(false, Ordering::Relaxed);
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::home,
            commands::get_date,
            commands::set_pause,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Tauri application");
}
