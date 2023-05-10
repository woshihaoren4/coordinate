use std::ops::Deref;
use std::sync::Arc;
use sqlx::{PgPool, Postgres};
use tonic::{Code, Request, Response, Status};
use wd_tools::PFOk;
use crate::app::entity;
use crate::proto;
use crate::proto::{CreateTaskRequest, CreateTaskResponse, SearchTasksRequest, SearchTasksResponse, TaskDetailRequest, TaskDetailResponse};

pub struct  TaskService{
    db_pool:Arc<PgPool>
}

impl TaskService {
    pub fn new(db_pool:Arc<PgPool>)->TaskService{
        TaskService{db_pool}
    }
}

#[tonic::async_trait]
impl proto::coordination_service_server::CoordinationService for TaskService{
    async fn create_task(&self, request: Request<CreateTaskRequest>) -> Result<Response<CreateTaskResponse>, Status> {
        //参数校验
        //存储
        let te = entity::TaskEntity::from(request.get_ref());
        if let Err(e) = te.create(self.db_pool.deref()).await {
            return Response::new(CreateTaskResponse{id:0,code:500,message:format!("insert failed:{}",e)}).ok()
        }
        Response::new(CreateTaskResponse{id:te.task_id,code:0,message:"success".into()}).ok()
    }

    async fn search_tasks(&self, request: Request<SearchTasksRequest>) -> Result<Response<SearchTasksResponse>, Status> {
        Err(Status::new(Code::Unknown,"todo"))
    }

    async fn task_detail(&self, request: Request<TaskDetailRequest>) -> Result<Response<TaskDetailResponse>, Status> {
        Err(Status::new(Code::Unknown,"todo"))
    }
}