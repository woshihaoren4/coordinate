mod task_entity;
mod node_entity;

use std::time::Duration;
use etcd_rs::{DeleteRequest, KeyRange, KeyValueOp, LeaseGrantRequest, LeaseOp, PutRequest, RangeRequest, TxnCmp};
use wd_tools::{PFErr, PFOk};
pub use task_entity::TaskEntity;
pub use node_entity::NodeEntity;
use crate::app::common;
use crate::infra::db;




#[tonic::async_trait]
pub trait EntityStore:Send+Sync{
    async fn create_task(&self,task_id:i64,task: &TaskEntity) ->anyhow::Result<()>;
    async fn get_task(&self,task_id:i64) ->anyhow::Result<TaskEntity>;

    async fn get_nodes(&self,task_id:i64) -> anyhow::Result<Vec<String>>;
    async fn register_nodes(&self,task_id:i64,node:&NodeEntity) ->anyhow::Result<()>;

    // async fn update_slot_info(&self,task_id:String,info:Box<dyn Entity>) ->anyhow::Result<()>;
}

#[tonic::async_trait]
pub trait GlobalLock:Send+Sync{
    async fn lock(&self,lock_key:String,value:String,timeout:Duration)->anyhow::Result<bool>;
    async fn unlock(&self,lock_key:String,value:String)->anyhow::Result<()>;
}

#[tonic::async_trait]
impl EntityStore for db::EtcdClient {
    async fn create_task(&self, task_id: i64, task: &TaskEntity) -> anyhow::Result<()> {
        let key = format!("{}/task/{}",common::DB_VERSION,task_id);
        let value = task.to_string();
        self.client.put((key,value)).await?;
        Ok(())
    }

    async fn get_task(&self, task_id: i64) -> anyhow::Result<TaskEntity> {
        let key = format!("{}/task/{}",common::DB_VERSION,task_id);
        let mut range = self.client.get(key).await?;
        if range.kvs.len() > 0 {
            let str = String::from_utf8(range.kvs.remove(0).value)?;
            TaskEntity::from(str).ok()
        }else{
            anyhow::anyhow!("record not found").err()
        }
    }

    async fn get_nodes(&self, task_id: i64) -> anyhow::Result<Vec<String>> {

        Ok(Vec::new())
    }

    async fn register_nodes(&self, task_id: i64, node: &NodeEntity) -> anyhow::Result<()> {
        Ok(())
    }

    // async fn update_slot_info(&self, task_id: String, info: Box<dyn Entity>) -> anyhow::Result<()> {
    //     Ok(())
    // }
}

#[tonic::async_trait]
impl GlobalLock for db::EtcdClient {
    async fn lock(&self, lock_key: String, value: String, timeout: Duration) -> anyhow::Result<bool> {
        let lease = self.client.grant_lease(timeout).await?;
        let key = format!("{}/lock/{}",common::DB_VERSION,lock_key);

        for i in 1..4{
            let request = etcd_rs::TxnRequest::new()
                .when_version(KeyRange::key(key.clone()), TxnCmp::Equal, 0)
                .and_then(PutRequest::new(key.clone(), value.clone()).lease(lease.id))
                .or_else( RangeRequest::from(KeyRange::key(key.clone())));

            let txn_resp = self.client.txn(request).await?;

            if txn_resp.succeeded {
                return Ok(true)
            }
            tokio::time::sleep(Duration::from_secs(i)).await;
        }
        Ok(false)
    }

    async fn unlock(&self, lock_key: String, value: String) -> anyhow::Result<()> {
        let key = format!("{}/lock/{}",common::DB_VERSION,lock_key);
        let request = etcd_rs::TxnRequest::new()
            .when_value(KeyRange::key(key.clone()), TxnCmp::Equal, value)
            .and_then(DeleteRequest::new(KeyRange::key(key.clone())))
            .or_else( RangeRequest::from(KeyRange::key(key)));

        let _ = self.client.txn(request).await?;Ok(())
    }
}