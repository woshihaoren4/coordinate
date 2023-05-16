use serde::{Deserialize, Serialize};
use crate::proto::{CreateTaskRequest};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskEntity{
    pub task_id : i64,
    pub app_id : i32,
    pub task_name: String,
    pub version : i32,

    pub dead_timeout_sec : i32,
    // pub r#type : i32,

    pub content:TaskContent,

    pub created_at : i64,
    pub updated_at : i64,
}

#[derive(Debug, Clone, Serialize, Deserialize,Default)]
pub struct TaskContent{
    slot_count : i32,
    node_max_count : i32,
    node_min_count : i32,
}


impl ToString for TaskEntity{
    fn to_string(&self) -> String {
        serde_json::to_string(self).expect("can not to here")
    }
}

impl From<CreateTaskRequest> for TaskEntity{
    fn from(value: CreateTaskRequest) -> Self {
        let slot = value.slot.unwrap();
        let nt = wd_tools::time::utc_timestamp();
        TaskEntity{
            task_id: wd_tools::snowflake_id(),
            app_id: value.app_id.unwrap_or(1),
            task_name: value.name,
            version: 0,
            dead_timeout_sec: value.strategy.unwrap().dead_timeout_sec,
            // r#type: value.mode.unwrap().into(),
            content: TaskContent{
                slot_count: slot.count,
                node_max_count: slot.node_max_count,
                node_min_count: slot.node_min_count},
            created_at: nt,
            updated_at: nt,
        }
    }
}