mod app;
mod proto;
mod cmd;
mod infra;
mod config;

#[tokio::main]
async fn main() {
    cmd::start().await;
}
