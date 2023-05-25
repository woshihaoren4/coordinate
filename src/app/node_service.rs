use std::sync::Arc;
use std::time::Duration;
use tonic::{Code, Request, Response, Status};
use crate::app::entity::{EntityStore, GlobalLock};
use crate::proto;
use crate::proto::{ExitTaskRequest, ExitTaskResponse, JoinTaskRequest, JoinTaskResponse, PingRequest, PingResponse, SlotDistributionsRequest, SlotDistributionsResponse};

pub struct NodeService {
    store : Arc<dyn EntityStore>,
    lock:Arc<dyn GlobalLock>
}

impl NodeService {
    pub fn new(store : Arc<dyn EntityStore>,lock:Arc<dyn GlobalLock>)-> NodeService {
        NodeService {store,lock}
    }
}

#[tonic::async_trait]
impl proto::node_service_server::NodeService for NodeService{
    async fn join_task(&self, request: Request<JoinTaskRequest>) -> Result<Response<JoinTaskResponse>, Status> {
        let tid = request.get_ref().task_id.to_string();
        let lock = wd_tools::uuid::v4();

        //加锁
        match self.lock.lock(tid.clone(),lock.clone(),Duration::from_secs(30)).await{
            Ok(result) => {
                if ! result {
                    server_err!(JoinTaskResponse,"系统繁忙",token:String::new())
                }
            }
            Err(err) => {
                server_err!(JoinTaskResponse,err,token:String::new())
            }
        }

        //todo

        //解锁
        if let Err(e) = self.lock.unlock(tid,lock).await{
            wd_log::log_error_ln!("join_task unlock：{}",e);
        }

        Err(Status::new(Code::Unknown,"todo"))
    }

    async fn exit_task(&self, request: Request<ExitTaskRequest>) -> Result<Response<ExitTaskResponse>, Status> {
        Err(Status::new(Code::Unknown,"todo"))
    }

    async fn ping(&self, request: Request<PingRequest>) -> Result<Response<PingResponse>, Status> {
        Err(Status::new(Code::Unknown,"todo"))
    }

    async fn slot_distributions(&self, request: Request<SlotDistributionsRequest>) -> Result<Response<SlotDistributionsResponse>, Status> {
        Err(Status::new(Code::Unknown,"todo"))
    }
}