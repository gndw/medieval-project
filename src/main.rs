mod app;
mod components;
mod content;
mod game;
mod http;

use app::App;
use content::startup as content_startup;
use game::date_advancing::tick as date_advancing_tick;
use game::settlement_simulation::tick as settlement_simulation_tick;
use http::startup as http_startup;

fn main() {
    let mut app = App::new();
    app.register_startup(content_startup);
    app.register_startup(http_startup);
    app.register_schedule("on_tick", date_advancing_tick);
    app.register_schedule("on_day", settlement_simulation_tick);
    app.run();
}