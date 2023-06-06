use crate::cmd::clean::CleanTask;
use crate::cmd::run::RunApplication;

mod run;
mod clean;

pub async fn start(){

    wd_run::AppEntity::new("distributed task scheduling center")
        .add_builder(RunApplication::default())
        .add_builder(CleanTask::default())
        .run().await
}