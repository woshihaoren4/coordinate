use crate::app::entity;
use crate::app::entity::{EntityStore, GlobalLock, NodeEntity, TaskEntity};
use crate::app::service::SoltRebalance;
use crate::config;
use crate::infra::exit::Exit;
use std::sync::Arc;
use std::time::Duration;
use wd_tools::pool::ParallelPool;
use wd_tools::PFOk;

pub const AUTOSCALING_LOCK: &'static str = "COORDINATION_AUTOSCALING_LOCK_GLOBALLY";

pub struct AutoScan {
    life: Exit,
    start_sec: i64,
    success_interval: Duration,
    try_interval: Duration,
    timeout: Duration,
    store: Arc<dyn EntityStore>,
    lock: Arc<dyn GlobalLock>,
    parallel: usize,
    // pool: ParallelPool,
}

impl AutoScan {
    pub fn new(
        cfg: config::Check,
        store: Arc<dyn EntityStore>,
        lock: Arc<dyn GlobalLock>,
        life: Exit,
    ) -> Self {
        let start_sec = cfg.start_sec;
        let success_interval = Duration::from_secs(cfg.success_interval_sec);
        let try_interval = Duration::from_secs(cfg.try_interval_sec);
        let timeout = Duration::from_secs(cfg.timeout);
        let parallel = cfg.parallel;
        Self {
            life,
            start_sec,
            success_interval,
            try_interval,
            timeout,
            store,
            lock,
            parallel,
        }
    }

    pub fn run(self) {
        tokio::spawn(self.start_inspection());
    }

    async fn start_inspection(self) {
        while self.life.status() <= 1 {
            //尝试加锁
            let val = wd_tools::uuid::v4();
            match self
                .lock
                .lock(AUTOSCALING_LOCK.to_string(), val.clone(), self.timeout, 1)
                .await
            {
                Ok(o) => {
                    if o {
                        // 拿锁成功，开始工作
                        wd_log::log_debug_ln!("auto scan try lock success, start scan");
                    } else {
                        //拿锁失败，等一会重试
                        wd_log::log_debug_ln!("auto scan try lock failed");
                        tokio::time::sleep(self.try_interval.clone()).await;
                        continue;
                    }
                }
                Err(e) => {
                    wd_log::log_error_ln!("auto scan -> try lock error:{}", e);
                    tokio::time::sleep(self.try_interval.clone()).await;
                    continue;
                }
            };
            //找出所有任务，并进行检查
            self.work().await;

            //解锁
            if let Err(e) = self.lock.unlock(AUTOSCALING_LOCK.to_string(), val).await {
                wd_log::log_error_ln!("auto scan unlock：{}", e);
            }
            tokio::time::sleep(self.success_interval.clone()).await;
            wd_log::log_debug_ln!("auto scan , rebalance over");
        }
    }

    async fn work(&self) {
        let result = self.store.tasks(0, 0).await;
        let ts = wd_log::res_error!(result;"scan tasks failed.").unwrap_or(vec![]);
        let pool = ParallelPool::new(self.parallel);
        for i in ts.into_iter() {
            let store = self.store.clone();
            let start_sec = self.start_sec;
            let lock = self.lock.clone();
            pool.launch(async move {
                let ok = match AutoScan::check(store.clone(), &i, start_sec).await {
                    Ok(o) => o,
                    Err(e) => {
                        wd_log::log_error_ln!(
                            "auto scan task, check task[{}] error:{}",
                            i.task_id,
                            e
                        );
                        return;
                    }
                };
                if !ok {
                    wd_log::log_debug_ln!("task[{}] normal", i.task_id);
                    return;
                }
                //检查不通过，则需要重排
                wd_log::log_debug_ln!("task[{}] check failed, need rebalance", i.task_id);
                let rebalance = async move {
                    let ns = AutoScan::get_nodes(store.clone(), i.task_id).await?;
                    let ns = SoltRebalance::new(&i.slot, ns).balance();

                    store.save_slot_detail(i.task_id, ns).await?;
                    Ok(())
                };
                let val = wd_tools::uuid::v4();
                if let Err(err) = entity::lock(
                    lock.clone(),
                    i.task_id.to_string(),
                    val,
                    Duration::from_secs(30),
                    rebalance,
                )
                .await
                {
                    wd_log::log_error_ln!("auto scan -> rebalance error:{}", err);
                } else {
                    wd_log::log_debug_ln!("task[{}] balance over", i.task_id);
                }
            })
            .await;
        }
        pool.wait_over().await;
    }
    async fn check(
        store: Arc<dyn EntityStore>,
        task: &TaskEntity,
        start_sec: i64,
    ) -> anyhow::Result<bool> {
        //任务创建一定时间内 不重排
        if task.created_at > wd_tools::time::utc_timestamp() - start_sec {
            return false.ok();
        }
        let ns = store.get_nodes(task.task_id).await?;
        // //节点全死
        // if ns.is_empty() {
        //     return true.ok();
        // }
        //节点变化 则重排
        let (_, ss) = store.get_slot_detail(task.task_id).await?;
        if ns.len() != ss.len() {
            return true.ok();
        }
        'lp: for i in ns.iter() {
            for j in ss.iter() {
                if i.code == j.code {
                    continue 'lp;
                }
            }
            return true.ok();
        }

        return false.ok();
    }
    async fn get_nodes(
        store: Arc<dyn EntityStore>,
        task_id: i64,
    ) -> anyhow::Result<Vec<NodeEntity>> {
        let ns = store.get_nodes(task_id).await?;
        if ns.is_empty() {
            return ns.ok();
        }
        let (_, mut ss) = store.get_slot_detail(task_id).await?;
        ss.retain(|x| ns.contains(x));
        ss.ok()
    }
}
