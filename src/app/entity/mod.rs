mod node_entity;
mod slot_dispatch_entity;
mod task_entity;

use crate::app::common;
use crate::infra::db;
use etcd_rs::{DeleteRequest, KeyRange, KeyValueOp, LeaseOp, PutRequest, RangeRequest, TxnCmp};
pub use node_entity::NodeEntity;
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;
pub use task_entity::*;
use wd_tools::{PFErr, PFOk};

#[tonic::async_trait]
pub trait EntityStore: Send + Sync {
    async fn create_task(&self, task_id: i64, task: &TaskEntity) -> anyhow::Result<()>;
    async fn get_task(&self, task_id: i64) -> anyhow::Result<TaskEntity>;
    async fn tasks(&self, page: i32, size: i32) -> anyhow::Result<Vec<TaskEntity>>;

    async fn get_nodes(&self, task_id: i64) -> anyhow::Result<Vec<NodeEntity>>;
    async fn register_nodes(&self, task_id: i64, node: &NodeEntity) -> anyhow::Result<()>;
    async fn remove_node(&self, task_id: i64, code: &str) -> anyhow::Result<()>;

    async fn get_slot_detail(&self, task_id: i64) -> anyhow::Result<Vec<NodeEntity>>;
    async fn save_slot_detail(&self, task_id: i64, ns: Vec<NodeEntity>) -> anyhow::Result<i64>;
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

#[tonic::async_trait]
impl EntityStore for db::EtcdClient {
    async fn create_task(&self, task_id: i64, task: &TaskEntity) -> anyhow::Result<()> {
        let key = format!("{}/task/{}", common::DB_VERSION, task_id);
        let value = task.to_string();
        self.client.put((key, value)).await?;
        Ok(())
    }

    async fn get_task(&self, task_id: i64) -> anyhow::Result<TaskEntity> {
        let key = format!("{}/task/{}", common::DB_VERSION, task_id);
        let mut range = self.client.get(key).await?;
        if range.kvs.len() > 0 {
            let str = String::from_utf8(range.kvs.remove(0).value)?;
            TaskEntity::from(str).ok()
        } else {
            anyhow::anyhow!("record not found").err()
        }
    }

    async fn tasks(&self, _page: i32, _size: i32) -> anyhow::Result<Vec<TaskEntity>> {
        let pre = format!("{}/task/", common::DB_VERSION);
        let rge = self.client.get(KeyRange::prefix(pre)).await?;
        let mut list = Vec::with_capacity(rge.kvs.len());
        for i in rge.kvs.iter() {
            list.push(TaskEntity::from(&i.value));
        }
        list.ok()
    }

    async fn get_nodes(&self, task_id: i64) -> anyhow::Result<Vec<NodeEntity>> {
        let pre = format!("{}/node/{}/", common::DB_VERSION, task_id);
        let rge = self.client.get(KeyRange::prefix(pre)).await?;
        let mut list = Vec::with_capacity(rge.kvs.len());
        for i in rge.kvs.iter() {
            list.push(NodeEntity::from(&i.value));
        }
        list.ok()
    }

    async fn register_nodes(&self, task_id: i64, node: &NodeEntity) -> anyhow::Result<()> {
        let key = format!(
            "{}/node/{}/{}",
            common::DB_VERSION,
            task_id,
            node.code.as_str()
        );
        let value = node.to_string();
        self.client.put((key, value)).await?;
        Ok(())
    }

    async fn remove_node(&self, task_id: i64, code: &str) -> anyhow::Result<()> {
        let key = format!("{}/node/{}/{}", common::DB_VERSION, task_id, code);
        self.client.delete(KeyRange::key(key)).await?;
        Ok(())
    }

    async fn get_slot_detail(&self, task_id: i64) -> anyhow::Result<Vec<NodeEntity>> {
        let key = format!("{}/slot/{}", common::DB_VERSION, task_id);
        let mut range = self.client.get(key).await?;
        if range.kvs.len() > 0 {
            let ns =
                serde_json::from_slice::<Vec<NodeEntity>>(range.kvs.remove(0).value.as_slice())?;
            Ok(ns)
        } else {
            Ok(Vec::new())
        }
    }

    async fn save_slot_detail(&self, task_id: i64, ns: Vec<NodeEntity>) -> anyhow::Result<i64> {
        let val = serde_json::to_vec(&ns)?;
        let key = format!("{}/slot/{}", common::DB_VERSION, task_id);
        let resp = self.client.put((key, val)).await?;
        Ok(resp.header.revision())
    }
}

#[tonic::async_trait]
impl GlobalLock for db::EtcdClient {
    async fn lock(
        &self,
        lock_key: String,
        value: String,
        timeout: Duration,
        again: u64,
    ) -> anyhow::Result<bool> {
        let lease = self.client.grant_lease(timeout).await?;
        let key = format!("{}/lock/{}", common::DB_VERSION, lock_key);

        for i in 0..again {
            let request = etcd_rs::TxnRequest::new()
                .when_version(KeyRange::key(key.clone()), TxnCmp::Equal, 0)
                .and_then(PutRequest::new(key.clone(), value.clone()).lease(lease.id))
                .or_else(RangeRequest::from(KeyRange::key(key.clone())));

            let txn_resp = self.client.txn(request).await?;

            if txn_resp.succeeded {
                return Ok(true);
            }
            tokio::time::sleep(Duration::from_secs(i + 1)).await;
        }
        Ok(false)
    }

    async fn unlock(&self, lock_key: String, value: String) -> anyhow::Result<()> {
        let key = format!("{}/lock/{}", common::DB_VERSION, lock_key);
        let request = etcd_rs::TxnRequest::new()
            .when_value(KeyRange::key(key.clone()), TxnCmp::Equal, value)
            .and_then(DeleteRequest::new(KeyRange::key(key.clone())))
            .or_else(RangeRequest::from(KeyRange::key(key)));

        let _ = self.client.txn(request).await?;
        Ok(())
    }
}
