mod config;

pub use config::*;
use std::path::Path;
use wd_tools::{PFErr, PFOk};

pub fn load_config_by_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Config> {
    match wd_run::load_config::<Config>(path) {
        Ok(o) => o.ok(),
        Err(e) => anyhow::anyhow!("{}", e).err(),
    }
}
