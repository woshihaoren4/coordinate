use futures::SinkExt;
use sqlx::{Executor, PgPool, Postgres, Transaction};
use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use sqlx::types::time::Time;
use crate::proto;
use crate::proto::{CreateTaskRequest, Strategy};
use crate::proto::create_task_request::Mode;

// #[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Debug, Clone)]
pub struct TaskEntity{
    pub task_id : i64,
    pub app_id : i32,
    pub task_name: String,
    pub version : i32,

    pub dead_timeout_sec : i32,
    pub r#type : i32,

    pub content: Json<TaskContent>,

    pub created_at : Time,
    pub updated_at : Time,
}

#[derive(Debug, Clone, Serialize, Deserialize,Default)]
pub struct TaskContent{
    slot_count : i32,
}

impl TaskEntity
{
    pub async fn create<'c, E>(&self,tx:E)->anyhow::Result<()>
        where
            E: Executor<'c, Database = Postgres>,
    {
        sqlx::query(r#"INSERT INTO tasks(task_id, app_id, task_name, "version", dead_timeout_sec, "type","content") VALUES($1, $2, $3, $4, $5, $6, $7);"#)
            .bind(&self.task_id)
            .bind(&self.app_id)
            .bind(&self.task_name)
            .bind(&self.version)
            .bind(&self.dead_timeout_sec)
            .bind(&self.r#type)
            .bind(&self.content)
            .execute(tx).await?;Ok(())
    }
}

impl From<&proto::CreateTaskRequest> for TaskEntity
{
    fn from(value: &CreateTaskRequest) -> TaskEntity {
         let mut te = TaskEntity{
             task_id: wd_tools::snowflake_id(),
             app_id: value.app_id.unwrap_or(1),
             task_name: value.name.clone(),
             version : 0,
             dead_timeout_sec: 60,
             content: Json(TaskContent::default()),
             r#type: 0,
             created_at: Time::MIDNIGHT,
             updated_at: Time::MIDNIGHT,
         };
        if let Some(ref s) =  value.strategy {
            te.dead_timeout_sec = s.dead_timeout_sec as i32;
        }
        if let Some(ref s) = value.mode {
            match s {
                Mode::Master(_) => te.r#type = 1,
                Mode::Slot(slot) => {
                    te.r#type = 2;
                    te.content.slot_count = slot.count;
                }
                Mode::HashRing(_) => te.r#type = 3,
            }
        }
        te
    }
}

// message Coordinator{
// int64 id = 1;
// int32 app_id = 2;
// string name = 3;
// int32 version = 4; //变更版本号
//
// repeated Node nodes = 50; //节点集
// Strategy strategy = 51;   //策略
//
//
// oneof mode{
// Master master = 100;
// Slot slot = 101;
// HashRing hash_ring = 102;
// };
// }