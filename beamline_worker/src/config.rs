use std::path::Path;

use anyhow::{bail, Context, Result};
use serde::Deserialize;


#[derive(Deserialize, Debug, Clone)]
pub struct Config
{
    pub redis_config: RedisConfig,
    pub control_clients: Vec<ControlClient>,
    pub ioc_config: IocConfig,
    /// Optional XRF live-map stream. Omit the section to disable the listener.
    #[serde(default)]
    pub xrf_stream: Option<XrfStreamConfig>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct XrfStreamConfig
{
    /// Hostname/IP of the XRF-Maps ZeroMQ PUB socket.
    pub host: String,
    /// TCP port of the XRF-Maps ZeroMQ PUB socket.
    pub port: u16,
    /// Subscription topic prefix; empty subscribes to all messages.
    #[serde(default)]
    pub topic: String,
    /// Byte width of XRF-Maps' real type: 4 (float) or 8 (double).
    #[serde(default = "default_real_bytes")]
    pub real_bytes: usize,
}

fn default_real_bytes() -> usize {
    4
}

#[derive(Deserialize, Debug, Clone)]
pub struct RedisConfig 
{
    pub conn_str: String,
    pub redis_cmd_queue: String,
    pub username: Option<String>,
    pub password: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ControlClient
{
    pub host: String,
    pub zmq_log_topic: String,
    pub beamline_id: String,
    pub protocol: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IocConfig
{
    /// Name of the Redis hash that all PV values are written into.
    pub hash_key: String,

    /// Seconds to wait for a PV to connect before retrying.
    #[serde(default = "default_connect_timeout")]
    pub connect_timeout_secs: u64,

    /// Seconds to back off after a disconnect before reconnecting.
    #[serde(default = "default_reconnect_backoff")]
    pub reconnect_backoff_secs: u64,

    /// The PVs to monitor, each paired with the label used as its Redis hash field.
    pub pvs: Vec<PvEntry>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PvEntry {
    /// Redis hash field key under which this PV's value is stored.
    pub label: String,
    /// The EPICS PV name to monitor.
    pub pv: String,
}



fn default_connect_timeout() -> u64 {
    10
}

fn default_reconnect_backoff() -> u64 {
    2
}

impl Config {
    /// Load and validate the configuration from a JSON file.
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("reading config file {}", path.display()))?;
        let cfg: Config = serde_json::from_str(&text)
            .with_context(|| format!("parsing JSON config {}", path.display()))?;
        cfg.validate()?;
        Ok(cfg)
    }

    fn validate(&self) -> Result<()> {
        if self.redis_config.conn_str.trim().is_empty() {
            bail!("`redis_url` must not be empty");
        }
        if self.ioc_config.hash_key.trim().is_empty() {
            bail!("`hash_key` must not be empty");
        }
        if self.ioc_config.pvs.is_empty() {
            bail!("`pvs` must contain at least one entry");
        }
        let mut seen = std::collections::HashSet::new();
        for entry in &self.ioc_config.pvs {
            if entry.label.trim().is_empty() {
                bail!("PV labels must not be empty (PV {:?})", entry.pv);
            }
            if entry.pv.trim().is_empty() {
                bail!("PV names must not be empty (label {:?})", entry.label);
            }
            if !seen.insert(entry.label.as_str()) {
                bail!(
                    "duplicate PV label {:?}; labels are Redis hash fields and must be unique",
                    entry.label
                );
            }
        }
        Ok(())
    }
}
