mod proto;
mod cmd;
mod app;
mod infra;
mod config;

#[tokio::main]
async fn main() {
    cmd::start().await;
}
