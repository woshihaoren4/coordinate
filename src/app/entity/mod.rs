mod node_entity;
mod slot_dispatch_entity;
mod task_entity;
mod dao;

pub use node_entity::NodeEntity;
pub use task_entity::*;

use std::future::Future;
use std::sync::Arc;
use std::time::Duration;
use etcd_rs::{KeyRange, KeyValueOp};
use crate::app::common;
use crate::infra::db;


#[tonic::async_trait]
pub trait EntityStore: Send + Sync {
    async fn create_task(&self, task_id: i64, task: &TaskEntity) -> anyhow::Result<()>;
    async fn get_task(&self, task_id: i64) -> anyhow::Result<TaskEntity>;
    async fn tasks(&self, page: i32, size: i32) -> anyhow::Result<Vec<TaskEntity>>;

    async fn get_nodes(&self, task_id: i64) -> anyhow::Result<Vec<NodeEntity>>;
    async fn node(&self, task_id: i64, code: String) -> anyhow::Result<NodeEntity>;
    async fn save_node(&self, task_id: i64, node: &NodeEntity) -> anyhow::Result<()>;
    async fn remove_node(&self, task_id: i64, code: &str) -> anyhow::Result<()>;

    async fn get_slot_detail(&self, task_id: i64) -> anyhow::Result<(i64, Vec<NodeEntity>)>;
    async fn save_slot_detail(&self, task_id: i64, ns: Vec<NodeEntity>) -> anyhow::Result<i64>;
    async fn get_slot_revision(&self, task_id: i64) -> anyhow::Result<i64>;
    // async fn update_slot_info(&self,task_id:String,info:Box<dyn Entity>) ->anyhow::Result<()>;
}

#[tonic::async_trait]
pub trait GlobalLock: Send + Sync {
    async fn lock(
        &self,
        lock_key: String,
        value: String,
        timeout: Duration,
        again: u64,
    ) -> anyhow::Result<bool>;
    async fn unlock(&self, lock_key: String, value: String) -> anyhow::Result<()>;
}

pub async fn lock<Out, Handle>(
    lock: Arc<dyn GlobalLock>,
    lock_key: String,
    value: String,
    timeout: Duration,
    handle: Handle,
) -> anyhow::Result<Out>
where
    Handle: Future<Output = anyhow::Result<Out>> + Send,
{
    //加锁
    let result = lock
        .lock(lock_key.clone(), value.clone(), timeout, 3)
        .await?;
    if !result {
        return Err(anyhow::anyhow!("lock try failed"));
    }

    let result = handle.await;

    //解锁
    if let Err(e) = lock.unlock(lock_key, value).await {
        wd_log::log_error_ln!("join_task unlock：{}", e);
    }

    return result;
}
pub async fn clean_tasks(client: db::EtcdClient) -> anyhow::Result<u64> {
    let pre = format!("{}/task/", common::DB_VERSION);
    let resp = client.client.delete(KeyRange::prefix(pre)).await?;
    Ok(resp.deleted)
}

