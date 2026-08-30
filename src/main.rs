mod app;
mod components;
mod content;
mod game;
mod http;

use app::App;
use content::startup as content_startup;
use game::date_advancing::tick as date_advancing_tick;
use http::startup as http_startup;

fn main() {
    let mut app = App::new();
    app.register_startup(content_startup);

    // The HTTP layer needs the pause flag as well as the world, so it is
    // registered through a closure rather than as a bare fn pointer.
    let is_tick_paused = app.pause_handle();
    app.register_startup(move |world| http_startup(world, is_tick_paused.clone()));

    app.register_tick(date_advancing_tick);
    app.run();
}
