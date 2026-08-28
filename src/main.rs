mod app;
mod components;
mod content;
mod http;

use std::net::SocketAddr;

use app::App;
use content::startup;

fn main() {
    let mut app = App::new();
    app.register_startup(startup);

    // Bind address for the HTTP server. Override with MEDIEVAL_HTTP_ADDR.
    let addr: SocketAddr = std::env::var("MEDIEVAL_HTTP_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:7777".into())
        .parse()
        .expect("invalid MEDIEVAL_HTTP_ADDR (expected e.g. 127.0.0.1:7777)");

    // Directory where the built SPA lives. Override with MEDIEVAL_STATIC_DIR.
    let static_dir: &'static str = Box::leak(
        std::env::var("MEDIEVAL_STATIC_DIR").unwrap_or_else(|_| "web/dist".into())
            .into_boxed_str(),
    );

    // Spawn the HTTP server on its own thread before entering the main loop.
    http::serve(app.world_handle(), addr, static_dir);

    app.run();
}
