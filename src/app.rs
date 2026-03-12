use std::net::SocketAddr;

use crate::api;
use crate::cli::{self, Command};

/// Run the application entrypoint (CLI or server) and return an exit code.
pub async fn run() -> i32 {
    let args: Vec<String> = std::env::args().collect();
    match cli::parse_args(&args) {
        Ok(Command::Serve) => {
            run_server().await;
            0
        }
        Ok(Command::Scan(args)) => {
            if let Err(err) = cli::run_scan(args) {
                eprintln!("{}", err);
                1
            } else {
                0
            }
        }
        Ok(Command::Stats(args)) => {
            if let Err(err) = cli::run_stats(args) {
                eprintln!("{}", err);
                1
            } else {
                0
            }
        }
        Err(message) => {
            eprintln!("{}", message);
            2
        }
    }
}

/// Start the HTTP server in serve mode.
async fn run_server() {
    let root = std::env::current_dir().expect("current dir");
    let state = api::state_from_root(root);
    let app = api::router(state);

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(3000);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("server failed");
}
