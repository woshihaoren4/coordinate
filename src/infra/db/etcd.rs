use etcd_rs::{Client, ClientConfig, Endpoint};
use wd_tools::PFOk;

pub struct EtcdClient {
    pub client: Client,
}

impl EtcdClient {
    pub async fn init(urls: Vec<String>) -> anyhow::Result<EtcdClient> {
        let eps: Vec<Endpoint> = urls.iter().map(|x| x.into()).collect();
        let client = Client::connect(ClientConfig::new(eps)).await?;
        EtcdClient { client }.ok()
    }
}
