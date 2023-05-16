mod task_entity;
mod node_entity;

use etcd_rs::KeyValueOp;
use wd_tools::PFOk;
pub use task_entity::TaskEntity;
pub use node_entity::NodeEntity;
use crate::infra::db;

const DB_VERSION:&'static str = "/api/v1";


#[tonic::async_trait]
pub trait EntityStore:Send+Sync{
    async fn create_task(&self,task_id:String,task: &TaskEntity) ->anyhow::Result<()>;
    async fn get_task(&self,task_id:String) ->anyhow::Result<String>;

    async fn get_nodes(&self,task_id:String) -> anyhow::Result<Vec<String>>;
    async fn register_nodes(&self,task_id:String,node:&NodeEntity) ->anyhow::Result<()>;

    // async fn update_slot_info(&self,task_id:String,info:Box<dyn Entity>) ->anyhow::Result<()>;
}

#[tonic::async_trait]
impl EntityStore for db::EtcdClient {
    async fn create_task(&self, task_id: String, task: &TaskEntity) -> anyhow::Result<()> {
        let key = format!("{}/task/{}",DB_VERSION,task_id);
        let value = task.to_string();
        self.client.put((key,value)).await?;
        Ok(())
    }

    async fn get_task(&self, task_id: String) -> anyhow::Result<String> {
        let mut range = self.client.get_by_prefix(task_id).await?;
        if range.count > 0 {
            let str = String::from_utf8(range.kvs.remove(0).value)?;str.ok()
        }else{
            String::new().ok()
        }
    }

    async fn get_nodes(&self, task_id: String) -> anyhow::Result<Vec<String>> {
        Ok(Vec::new())
    }

    async fn register_nodes(&self, task_id: String, node: &NodeEntity) -> anyhow::Result<()> {
        Ok(())
    }

    // async fn update_slot_info(&self, task_id: String, info: Box<dyn Entity>) -> anyhow::Result<()> {
    //     Ok(())
    // }
}