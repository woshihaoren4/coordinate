use crate::app::entity::{EntityStore, GlobalLock};
use crate::app::{entity, service};
use crate::proto;
use crate::proto::{
    ExitTaskRequest, ExitTaskResponse, JoinTaskRequest, JoinTaskResponse, PingRequest,
    PingResponse, SlotDistributionsRequest, SlotDistributionsResponse,
};
use std::sync::Arc;
use std::time::Duration;
use tonic::{Request, Response, Status};

pub struct NodeService {
    store: Arc<dyn EntityStore>,
    lock: Arc<dyn GlobalLock>,
}

impl NodeService {
    pub fn new(store: Arc<dyn EntityStore>, lock: Arc<dyn GlobalLock>) -> NodeService {
        NodeService { store, lock }
    }
}

#[tonic::async_trait]
impl proto::node_service_server::NodeService for NodeService {
    async fn join_task(
        &self,
        request: Request<JoinTaskRequest>,
    ) -> Result<Response<JoinTaskResponse>, Status> {
        let tid = request.get_ref().task_id;
        let store = self.store.clone();
        let node = entity::NodeEntity::from(request.into_inner());

        let handle = async move {
            let task = store.get_task(tid).await?;
            let ns = store.get_slot_detail(tid).await?;
            for i in ns.iter() {
                if i.code == node.code {
                    //已存在则不重复插入
                    return Ok(task.secret);
                }
            }

            self.store.save_node(tid, &node).await?;

            let ns = service::SoltRebalance::new(&task.slot, ns).join(node);

            self.store.save_slot_detail(tid, ns).await?;

            Ok(task.secret)
        };

        let lock = wd_tools::uuid::v4();
        let result = entity::lock(
            self.lock.clone(),
            tid.to_string(),
            lock,
            Duration::from_secs(30),
            handle,
        )
        .await;

        match result {
            Ok(token) => {
                success!(JoinTaskResponse, token: token)
            }
            Err(err) => {
                server_err!(JoinTaskResponse, err, token: String::new())
            }
        }
    }

    async fn exit_task(
        &self,
        request: Request<ExitTaskRequest>,
    ) -> Result<Response<ExitTaskResponse>, Status> {
        let tid = request.get_ref().task_id;
        let code = request.get_ref().code.clone();
        let store = self.store.clone();

        let handle = async move {
            let task = store.get_task(tid).await?;
            let ns = store.get_slot_detail(tid).await?;
            self.store.remove_node(tid, code.as_str()).await?;
            let ns = service::SoltRebalance::new(&task.slot, ns).remove(code);
            self.store.save_slot_detail(tid, ns).await?;
            Ok(())
        };

        let lock = wd_tools::uuid::v4();
        let result = entity::lock(
            self.lock.clone(),
            tid.to_string(),
            lock,
            Duration::from_secs(30),
            handle,
        )
        .await;

        match result {
            Ok(_) => {
                success!(ExitTaskResponse,balance_success:true)
            }
            Err(err) => {
                server_err!(ExitTaskResponse,err,balance_success:false)
            }
        }
    }

    async fn ping(&self, request: Request<PingRequest>) -> Result<Response<PingResponse>, Status> {
        let task_id = request.get_ref().task_id;
        let code = request.into_inner().code;

        let node = self.store.node(task_id,code).await;
        let mut node = match node {
            Ok(o)=> o,
            Err(e)=> server_err!(PingResponse,e,version:0)
        };

        let version = match self.store.get_slot_revision(task_id).await {
            Ok(ver) => ver,
            Err(err) => server_err!(PingResponse,err,version:0)
        };

        node.slot_version = version;
        node.last_ping_time = wd_tools::time::utc_timestamp();

        if let Err(err) = self.store.save_node(task_id,&node).await {
            server_err!(PingResponse,err,version:0)
        }

        success!(PingResponse,version:version)
    }

    async fn slot_distributions(
        &self,
        request: Request<SlotDistributionsRequest>,
    ) -> Result<Response<SlotDistributionsResponse>, Status> {
        let tid = request.get_ref().task_id;
        let code = request.get_ref().node_code.clone();

        let ns = match self.store.get_slot_detail(tid).await {
            Ok(o) => o,
            Err(e) => server_err!(SlotDistributionsResponse, e, tags: vec![]),
        };

        for i in ns.into_iter() {
            if i.code == code {
                success!(SlotDistributionsResponse,tags:i.tags)
            }
        }
        bad_request!(
            SlotDistributionsResponse,
            format!("node not found"),
            tags: vec![]
        )
    }
}
