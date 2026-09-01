mod app;
mod components;
mod content;
mod game;
mod http;

use app::App;

fn main() {
    let mut app = App::new();
    app.register_startup(content::startup);
    app.register_startup(http::startup);
    app.register_schedule("on_tick", game::date_advancing::tick);
    app.register_schedule("on_day", game::workplace_producing::on_day);
    app.run();
}