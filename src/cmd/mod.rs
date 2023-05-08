use crate::cmd::run::RunApplication;

mod run;

pub async fn start(){

    wd_run::AppEntity::new("distributed task scheduling center")
        .add_builder(RunApplication::default())
        .run().await
}