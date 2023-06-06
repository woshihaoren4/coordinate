use serde::{Serialize,Deserialize};
use crate::proto::{JoinTaskRequest};

#[derive(Debug, Clone, Serialize, Deserialize,Default)]
pub struct NodeEntity{
    // pub id : i64,
    pub code : String,
    pub status: i32,
    pub tags: Vec<i32>,
    pub last_ping_time:i64,

    pub created_at : i64,
}



impl ToString for NodeEntity{
    fn to_string(&self) -> String {
        serde_json::to_string(self).expect("can not to here")
    }
}

impl From<JoinTaskRequest> for NodeEntity {
    fn from(value: JoinTaskRequest) -> Self {
        let t =  wd_tools::time::utc_timestamp();
        NodeEntity{
            // id: wd_tools::snowflake_id(),
            code: value.code,
            status: 1,
            tags: vec![],
            last_ping_time: t,
            created_at: t,
        }
    }
}