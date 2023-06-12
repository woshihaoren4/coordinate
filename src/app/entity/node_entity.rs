use crate::proto::{JoinTaskRequest, Node};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NodeEntity {
    // pub id : i64,
    pub code: String,
    pub addr: String,
    pub status: i32,
    pub tags: Vec<i32>,
    pub last_ping_time: i64,
    pub slot_version: i64,

    pub created_at: i64,
}

impl ToString for NodeEntity {
    fn to_string(&self) -> String {
        serde_json::to_string(self).expect("can not to here")
    }
}
impl<T: AsRef<[u8]>> From<T> for NodeEntity {
    fn from(value: T) -> Self {
        serde_json::from_slice::<NodeEntity>(value.as_ref()).unwrap_or(Default::default())
    }
}
impl From<JoinTaskRequest> for NodeEntity {
    fn from(value: JoinTaskRequest) -> Self {
        let t = wd_tools::time::utc_timestamp();
        NodeEntity {
            // id: wd_tools::snowflake_id(),
            code: value.code,
            addr: value.addr,
            status: 1,
            tags: vec![],
            last_ping_time: t,
            created_at: t,
            slot_version:0,
        }
    }
}

impl Into<Node> for NodeEntity {
    fn into(self) -> Node {
        Node {
            code: self.code,
            addr: self.addr,
            status: self.status,
            last_ping_time: self.last_ping_time,
        }
    }
}
