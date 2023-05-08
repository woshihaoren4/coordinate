#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TaskEntity{
    task_id : String,
    app_id : i32,
    task_name: String,
    version : i32,

    dead_timeout_sec : i32,
    r#type : i32,

    created_at : i64,
    updated_at : i64,
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