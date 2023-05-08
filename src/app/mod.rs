use std::time::Duration;
use crate::config::Config;
use crate::infra::exit::Exit;
use crate::infra::middle::CustomInterceptor;
use crate::proto;

mod task_service;
mod node_service;
mod middles;

pub async fn server(cfg:Config,exit:Exit)->anyhow::Result<()>{
    let ts = task_service::TaskService::new();
    let ns = node_service::NodeService::new();
    let mid_log = middles::MiddleLog{};
    let mid_exit = middles::MiddleExit::new(exit);

    let layer = tower::ServiceBuilder::new()
        .timeout(Duration::from_secs(300))
        // .concurrency_limit(1000)
        .layer(CustomInterceptor::new(mid_exit))
        .layer(CustomInterceptor::new(mid_log))
        .into_inner();

    tonic::transport::Server::builder()
        .layer(layer)
        .add_service(proto::coordination_service_server::CoordinationServiceServer::new(ts))
        .add_service( proto::node_service_server::NodeServiceServer::new(ns))
        .serve("127.0.0.1:666".parse().unwrap()).await?;Ok(())
}



