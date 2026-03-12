use ssd_navigator::app;

#[tokio::main]
async fn main() {
    let code = app::run().await;
    if code != 0 {
        std::process::exit(code);
    }
}
