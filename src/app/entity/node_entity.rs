use serde::{Serialize,Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeEntity{
    id : i64,
    code : String,
    status: i32,
    tags: Vec<i32>,
    last_ping_time:i64,

    created_at : i64,
}



impl ToString for NodeEntity{
    fn to_string(&self) -> String {
        serde_json::to_string(self).expect("can not to here")
    }
}