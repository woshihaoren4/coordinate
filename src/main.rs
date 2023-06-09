mod app;
mod cmd;
mod config;
mod infra;
mod proto;

#[tokio::main]
async fn main() {
    cmd::start().await;
}
