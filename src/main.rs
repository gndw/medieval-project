mod app;
mod components;
mod content;

use app::App;
use content::startup;

fn main() {
    let mut app = App::new();
    app.register_startup(startup);
    app.run();
}
