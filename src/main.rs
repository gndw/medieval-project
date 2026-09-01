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
    let mut app = App::new();
    app.register_startup(content::startup);
    app.register_schedule("on_tick", game::date_advancing::tick);
    app.register_schedule("on_day", game::workplace_producing::on_day);

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

            // Drain startups (loads content into the world).
            app.run_startups();

            // Spawn the tick loop on its own OS thread.
            let stop = Arc::clone(&tick_stop);
            std::thread::Builder::new()
                .name("medieval-tick".into())
                .spawn(move || {
                    App::run_tick_loop(
                        app.state.clone(),
                        app.world.clone(),
                        app.schedules.clone(),
                        stop,
                    );
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
