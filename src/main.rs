mod app;
mod components;
mod content;
mod http;

use app::App;
use content::startup as content_startup;
use http::startup as http_startup;

fn main() {
    let mut app = App::new();
    app.register_startup(content_startup);
    app.register_startup(http_startup);
    app.run();
}
