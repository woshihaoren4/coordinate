use std::sync::Arc;
use std::sync::atomic::{AtomicIsize, Ordering};
use std::time::Duration;

#[derive(Debug,Default,Clone)]
pub struct Exit {
    status: Arc<AtomicIsize>, // 0:初始 1:开始工作 2:工作开始结束 3:开始退出
    task_count: Arc<AtomicIsize>, // 任务计数器
}

impl Exit {
    pub fn start(&self){
        self.status.store(1,Ordering::Relaxed);
    }
    #[allow(dead_code)]
    pub fn over(&self){
        self.status.store(3,Ordering::Relaxed);
    }
    pub fn add_task(&self){
        self.task_count.fetch_add(1,Ordering::Relaxed);
    }
    pub fn sub_task(&self){
        self.task_count.fetch_sub(1,Ordering::Relaxed);
    }
    pub fn status(&self)->isize{
        self.status.load(Ordering::Relaxed)
    }
    #[allow(dead_code)]
    pub async fn wait_exit(&self,timeout:Duration)->anyhow::Result<()>{
        if self.status.load(Ordering::Relaxed) == 0 {
            return Ok(())
        }
        self.status.store(2,Ordering::Relaxed);

        let sleep = tokio::time::sleep(timeout);
        tokio::pin!(sleep);

        let status = self.status.clone();
        let wait = async move {
            while status.load(Ordering::Relaxed) <= 2 {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        };

        tokio::select! {
            _ = &mut sleep =>{
                return Err(anyhow::anyhow!("exit failed,timeout"))
            },
            _ = wait =>{
                return Ok(())
            }
        }
    }
    pub async fn wait_task_complete_exit(&self,timeout:Duration)->anyhow::Result<()>{
        if self.status.load(Ordering::Relaxed) == 0 {
            return Ok(())
        }
        self.status.store(2,Ordering::Relaxed);

        let sleep = tokio::time::sleep(timeout);
        tokio::pin!(sleep);

        let count = self.task_count.clone();
        let wait = async move {
            while count.load(Ordering::Relaxed) != 0 {
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        };

        tokio::select! {
            _ = &mut sleep =>{
                return Err(anyhow::anyhow!("exit failed,timeout"))
            },
            _ = wait =>{
                return Ok(())
            }
        }
    }
}