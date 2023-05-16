use tonic::{Code, Request, Response, Status};
use crate::proto;
use crate::proto::{ExitTaskRequest, ExitTaskResponse, JoinTaskRequest, JoinTaskResponse, PingRequest, PingResponse, SlotDistributionsRequest, SlotDistributionsResponse};

pub struct NodeService {
}

impl NodeService {
    pub fn new()-> NodeService {
        NodeService {}
    }
}

#[tonic::async_trait]
impl proto::node_service_server::NodeService for NodeService{
    async fn join_task(&self, request: Request<JoinTaskRequest>) -> Result<Response<JoinTaskResponse>, Status> {


        //查询所有节点信息
        //节点重平衡
        //解锁
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