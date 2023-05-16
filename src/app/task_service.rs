use std::sync::Arc;
use tonic::{Code, Request, Response, Status};
use crate::app::entity::{EntityStore, TaskEntity};
use crate::proto;
use crate::proto::{CreateTaskRequest, CreateTaskResponse, SearchTasksRequest, SearchTasksResponse, TaskDetailRequest, TaskDetailResponse};

pub struct  TaskService{
    store : Arc<dyn EntityStore>
}

impl TaskService {
    pub fn new(store : Arc<dyn EntityStore>)->TaskService{
        TaskService{store}
    }
}



#[tonic::async_trait]
impl proto::coordination_service_server::CoordinationService for TaskService{
    async fn create_task(&self, request: Request<CreateTaskRequest>) -> Result<Response<CreateTaskResponse>, Status> {
        if request.get_ref().slot.is_none() {
            bad_request!(CreateTaskResponse,format!("request slot is nil"),id:0)
        }
        if request.get_ref().strategy.is_none() {
            bad_request!(CreateTaskResponse,format!("request strategy is nil"),id:0)
        }

        let task = TaskEntity::from(request.into_inner());
        match self.store.create_task(task.task_id.clone().to_string(), &task).await{
            Ok(_) => success!(CreateTaskResponse,id:task.task_id),
            Err(e) => server_err!(CreateTaskResponse,e,id:0),
        }
    }

    async fn search_tasks(&self, request: Request<SearchTasksRequest>) -> Result<Response<SearchTasksResponse>, Status> {
        Err(Status::new(Code::Unknown,"todo"))
    }

    async fn task_detail(&self, request: Request<TaskDetailRequest>) -> Result<Response<TaskDetailResponse>, Status> {
        Err(Status::new(Code::Unknown,"todo"))
    }
}