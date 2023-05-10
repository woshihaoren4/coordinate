use std::sync::Arc;
use std::time::Duration;
use wd_run::{Context, TaskBuild,Task, TaskInfo};
use wd_tools::PFArc;
use crate::{app, config};
use crate::config::Config;
use crate::infra::db;
use crate::infra::exit::Exit;

#[derive(Debug,Default,Clone)]
pub struct RunApplication{

}

#[wd_run::async_trait]
impl TaskBuild for RunApplication{
    fn args(&self) -> TaskInfo {
        TaskInfo::new("run","run coordination server")
            .register_arg("-c","./src/config/dev_config.toml","config file path")
    }

    async fn build(&mut self, mut ctx: Context) -> Arc<dyn Task> {
        let config_file = ctx.load::<String>("c").unwrap();
        wd_log::log_debug_ln!("config path:{}",config_file);

        let cfg = wd_log::res_panic!(config::load_config_by_file(config_file);"load config failed");
        wd_log::log_debug_ln!("config load success --->{}",cfg.to_string());

        let life = Exit::default();
        RunEntity {life,cfg  }.arc()
    }
}

pub struct RunEntity{
    life: Exit,
    cfg:Config,
}

#[wd_run::async_trait]
impl Task for RunEntity {
    async fn run(&self) -> anyhow::Result<()> {
        self.life.start();
        let db = db::init_pgsql(self.cfg.pgcfg.clone()).await.expect("connection pgsql failed");
        let res = app::server(self.cfg.clone(),self.life.clone(),db.arc()).await;
        res
    }

    async fn exit(&self) -> anyhow::Result<()> {
        self.life.wait_task_complete_exit(Duration::from_secs(5)).await
    }
}