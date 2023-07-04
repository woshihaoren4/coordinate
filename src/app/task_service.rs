use crate::app::entity::{EntityStore, TaskEntity};
use crate::proto;
use crate::proto::{CreateTaskRequest, CreateTaskResponse, SearchTasksRequest, SearchTasksResponse, Task, TaskDeleteRequest, TaskDeleteResponse, TaskDetailRequest, TaskDetailResponse};
use std::sync::Arc;
use tonic::{Request, Response, Status};

pub struct TaskService {
    store: Arc<dyn EntityStore>,
}

impl TaskService {
    pub fn new(store: Arc<dyn EntityStore>) -> TaskService {
        TaskService { store }
    }
}

#[tonic::async_trait]
impl proto::task_service_server::TaskService for TaskService {
    async fn create_task(
        &self,
        request: Request<CreateTaskRequest>,
    ) -> Result<Response<CreateTaskResponse>, Status> {
        if request.get_ref().slot.is_none() {
            bad_request!(CreateTaskResponse,format!("request slot is nil"),id:0)
        }
        if request.get_ref().strategy.is_none() {
            bad_request!(CreateTaskResponse,format!("request strategy is nil"),id:0)
        }

        let task = TaskEntity::from(request.into_inner());
        match self.store.create_task(task.task_id, &task).await {
            Ok(_) => success!(CreateTaskResponse,id:task.task_id),
            Err(e) => server_err!(CreateTaskResponse,e,id:0),
        }
    }

    async fn search_tasks(
        &self,
        _request: Request<SearchTasksRequest>,
    ) -> Result<Response<SearchTasksResponse>, Status> {
        let tasks = match self.store.tasks(0, 0).await {
            Ok(o) => o,
            Err(e) => server_err!(SearchTasksResponse, e, tasks: vec![]),
        };
        let mut list = Vec::with_capacity(tasks.len());
        for i in tasks.into_iter() {
            list.push(i.into());
        }
        success!(SearchTasksResponse, tasks: list)
    }

    async fn task_detail(
        &self,
        request: Request<TaskDetailRequest>,
    ) -> Result<Response<TaskDetailResponse>, Status> {
        let tid = request.get_ref().task_id;
        // wd_log::log_debug_ln!("查询任务详情：{} {}",request.get_ref().task_id,tid);
        let mut task = match self.store.get_task(tid).await {
            Ok(t) => <TaskEntity as Into<Task>>::into(t),
            Err(e) => server_err!(TaskDetailResponse, e, task: None),
        };
        let ns = match self.store.get_nodes(task.id).await {
            Ok(o) => o,
            Err(e) => server_err!(TaskDetailResponse, e, task: None),
        };
        let mut list = Vec::with_capacity(ns.len());
        for i in ns.into_iter() {
            list.push(i.into())
        }
        task.nodes = list;
        success!(TaskDetailResponse, task: Some(task))
    }

    async fn task_delete(&self, _request: Request<TaskDeleteRequest>) -> Result<Response<TaskDeleteResponse>, Status> {
        server_err!(TaskDeleteResponse,"not delete task",)
    }
}
