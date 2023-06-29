use etcd_rs::{Client, ClientConfig, Endpoint};
use wd_tools::PFOk;
use crate::config;

pub struct EtcdClient {
    pub client: Client,
}

impl EtcdClient {
    pub async fn init(cfg : config::Etcd) -> anyhow::Result<EtcdClient> {
        let mut etcd_cfg = ClientConfig::new(cfg.endpoints.iter().map(|x| x.into()).collect::<Vec<Endpoint>>());
        if !cfg.passwd.is_empty() && !cfg.user.is_empty() {
            etcd_cfg = etcd_cfg.auth(cfg.user,cfg.passwd);
        }
        let client = Client::connect(etcd_cfg).await?;
        EtcdClient { client }.ok()
    }
}
