use tonic::{Code, Request, Response, Status};
use crate::proto;
use crate::proto::{CreateTaskRequest, CreateTaskResponse, SearchTasksRequest, SearchTasksResponse, TaskDetailRequest, TaskDetailResponse};

pub struct  TaskService{}

impl TaskService {
    pub fn new()->TaskService{
        TaskService{}
    }
}

#[tonic::async_trait]
impl proto::coordination_service_server::CoordinationService for TaskService{
    async fn create_task(&self, request: Request<CreateTaskRequest>) -> Result<Response<CreateTaskResponse>, Status> {
        Err(Status::new(Code::Unknown,"todo"))
    }

    async fn search_tasks(&self, request: Request<SearchTasksRequest>) -> Result<Response<SearchTasksResponse>, Status> {
        Err(Status::new(Code::Unknown,"todo"))
    }

    async fn task_detail(&self, request: Request<TaskDetailRequest>) -> Result<Response<TaskDetailResponse>, Status> {
        Err(Status::new(Code::Unknown,"todo"))
    }
}