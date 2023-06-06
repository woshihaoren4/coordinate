use serde::{Deserialize, Serialize};
use wd_tools::PFSome;
use crate::proto::{CreateTaskRequest, Slot, Strategy, Task};

#[derive(Debug, Clone, Serialize, Deserialize,Default)]
pub struct TaskEntity{
    pub task_id : i64,
    pub app_id : i32,
    pub task_name: String,
    pub version : i32,
    pub secret : String,
    pub dead_timeout_sec : i32,
    // pub r#type : i32,

    pub slot: TaskSlot,


    pub created_at : i64,
    pub updated_at : i64,
}

#[derive(Debug, Clone, Serialize, Deserialize,Default)]
pub struct TaskSlot {
    pub slot_count : i32,
    pub node_max_count : i32,
    pub node_min_count : i32,
}


impl ToString for TaskEntity{
    fn to_string(&self) -> String {
        serde_json::to_string(self).expect("can not to here")
    }
}

impl<T: AsRef<[u8]>> From<T> for TaskEntity {
    fn from(value: T) -> Self {
        serde_json::from_slice::<TaskEntity>(value.as_ref()).unwrap_or(Default::default())
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
            secret : wd_tools::uuid::v4(),
            dead_timeout_sec: value.strategy.unwrap().dead_timeout_sec,
            // r#type: value.mode.unwrap().into(),
            slot: TaskSlot {
                slot_count: slot.count,
                node_max_count: slot.node_max_count,
                node_min_count: slot.node_min_count},
            created_at: nt,
            updated_at: nt,
        }
    }
}

impl Into<Task> for TaskEntity {
    fn into(self) -> Task {
        Task{
            id: self.task_id,
            app_id: self.app_id,
            name: self.task_name,
            version: self.version,
            nodes: vec![],
            strategy: Strategy{ dead_timeout_sec: self.dead_timeout_sec }.some(),
            slot: Slot{
                count: self.slot.slot_count,
                slot_alloc: vec![],
                node_max_count: self.slot.node_max_count,
                node_min_count: self.slot.node_max_count,
            }.some(),
        }
    }
}