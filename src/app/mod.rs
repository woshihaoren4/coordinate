#[macro_use]
pub mod common;

pub mod entity;
mod middles;
mod node_service;
pub mod service;
mod task_service;

use crate::app::entity::{EntityStore, GlobalLock};
use crate::config::Config;
use crate::infra::exit::Exit;
use crate::infra::middle::CustomInterceptor;
use crate::proto;
use std::sync::Arc;
use std::time::Duration;

pub async fn server(
    cfg: Config,
    exit: Exit,
    store: Arc<dyn EntityStore>,
    lock: Arc<dyn GlobalLock>,
) -> anyhow::Result<()> {
    let ts = task_service::TaskService::new(store.clone());
    let ns = node_service::NodeService::new(store.clone(), lock);
    let mid_log = middles::MiddleLog {};
    let mid_exit = middles::MiddleExit::new(exit);

    let layer = tower::ServiceBuilder::new()
        .timeout(Duration::from_secs(300))
        // .concurrency_limit(1000)
        .layer(CustomInterceptor::new(mid_exit))
        .layer(CustomInterceptor::new(mid_log))
        .into_inner();

    wd_log::log_debug_ln!("server start lister :{}", cfg.server.host_port);
    tonic::transport::Server::builder()
        .layer(layer)
        .add_service(proto::task_service_server::TaskServiceServer::new(ts))
        .add_service(proto::node_service_server::NodeServiceServer::new(ns))
        .serve(cfg.server.host_port.parse().unwrap())
        .await?;
    Ok(())
}
