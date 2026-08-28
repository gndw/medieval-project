mod app;
mod components;
mod content;

use app::App;
use content::load;

fn main() {
    let mut content = load();
    let mut app = App::New();
    content.Startup(&mut app.world);
    app.Run();
}
