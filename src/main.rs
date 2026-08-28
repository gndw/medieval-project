mod app;
mod components;
mod content;

use app::App;
use content::Startup;

fn main() {
    let mut app = App::New();
    app.RegisterStartup(Startup);
    app.Run();
}
