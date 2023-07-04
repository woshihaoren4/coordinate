use std::time::Duration;
use etcd_rs::{DeleteRequest, KeyRange, KeyValueOp, LeaseOp, PutRequest, RangeRequest, TxnCmp};
use wd_tools::{PFErr, PFOk};
use crate::app::common;
use crate::app::entity::{EntityStore, GlobalLock, NodeEntity, TaskEntity};
use crate::infra::db;

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

    async fn node(&self, task_id: i64, code: String) -> anyhow::Result<NodeEntity> {
        let key = format!("{}/node/{}/{}", common::DB_VERSION, task_id, code);
        let resp = self.client.get(key).await?;
        if resp.kvs.len() <= 0 {
            return anyhow::anyhow!("record not found").err();
        }
        NodeEntity::from(&resp.kvs[0].value).ok()
    }

    async fn save_node(&self, task_id: i64, node: &NodeEntity) -> anyhow::Result<()> {
        let lease = self.client.grant_lease(Duration::from_secs(60 * 3)).await?;
        let key = format!(
            "{}/node/{}/{}",
            common::DB_VERSION,
            task_id,
            node.code.as_str()
        );
        let value = node.to_string();
        self.client
            .put(PutRequest::from((key, value)).lease(lease.id))
            .await?;
        Ok(())
    }

    async fn remove_node(&self, task_id: i64, code: &str) -> anyhow::Result<()> {
        let key = format!("{}/node/{}/{}", common::DB_VERSION, task_id, code);
        self.client.delete(KeyRange::key(key)).await?;
        Ok(())
    }

    async fn get_slot_detail(&self, task_id: i64) -> anyhow::Result<(i64, Vec<NodeEntity>)> {
        let key = format!("{}/slot/{}", common::DB_VERSION, task_id);
        let range = self.client.get(key).await?;
        if range.kvs.len() > 0 {
            let ns = serde_json::from_slice::<Vec<NodeEntity>>(range.kvs[0].value.as_slice())?;
            Ok((range.kvs[0].version, ns))
        } else {
            Ok((0, Vec::new()))
        }
    }

    async fn save_slot_detail(&self, task_id: i64, ns: Vec<NodeEntity>) -> anyhow::Result<i64> {
        let val = serde_json::to_vec(&ns)?;
        let key = format!("{}/slot/{}", common::DB_VERSION, task_id);
        let resp = self.client.put((key, val)).await?;
        Ok(resp.header.revision())
    }

    async fn get_slot_revision(&self, task_id: i64) -> anyhow::Result<i64> {
        let key = format!("{}/slot/{}", common::DB_VERSION, task_id);
        let resp = self.client.get(key).await?;
        if resp.kvs.len() <= 0 {
            return anyhow::anyhow!("slot revision not found").err();
        }
        resp.kvs[0].version.ok()
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
