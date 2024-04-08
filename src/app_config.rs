use std::{collections::HashMap, sync::OnceLock};

use anyhow::anyhow;
use tracing::debug;

type AppConfigMap = HashMap<String, AppConfig>;

static APP_CONFIGS: OnceLock<AppConfigMap> = OnceLock::new();

#[derive(Debug, Clone)]
pub enum AppConfig {
    Port(u16),
    ReplicaOf(String, String),
    MasterReplId(String),
    MasterReplOffset(u128),
}
impl AppConfig {
    pub(crate) fn get_port() -> u16 {
        let args = APP_CONFIGS.get_or_init(|| Self::init().unwrap());
        args.get("--port")
            .map(|v| match v {
                AppConfig::Port(port) => *port,
                _ => 6_379_u16,
            })
            .unwrap_or(6_379_u16)
    }
    pub(crate) fn is_master() -> bool {
        let args = APP_CONFIGS.get_or_init(|| Self::init().unwrap());
        args.get("--replicaof").is_none()
    }
    pub(crate) fn get_replicaof() -> Option<(String, String)> {
        let args = APP_CONFIGS.get_or_init(|| Self::init().unwrap());
        let Some(AppConfig::ReplicaOf(host, port)) = args.get("--replicaof") else {
            return None;
        };
        Some((host.to_owned(), port.to_owned()))
    }
    pub(crate) fn get_master_replid() -> String {
        let args = APP_CONFIGS.get_or_init(|| Self::init().unwrap());
        args.get("$$master_replid")
            .map(|v| match v {
                AppConfig::MasterReplId(replid) => replid.clone(),
                _ => "".to_string(),
            })
            .unwrap_or("".to_string())
    }
    pub(crate) fn get_master_repl_offset() -> u128 {
        let args = APP_CONFIGS.get_or_init(|| Self::init().unwrap());
        args.get("$$master_repl_offset")
            .map(|v| match v {
                AppConfig::MasterReplOffset(offset) => *offset,
                _ => 0,
            })
            .unwrap_or(0)
    }
    pub fn init() -> anyhow::Result<AppConfigMap> {
        let mut args = std::env::args();
        args.next();
        let mut map = HashMap::new();
        while let Some(arg) = args.next() {
            debug!("Processing arg: {}", arg);
            let cli_arg = match arg.as_str() {
                "--port" => match args.next() {
                    Some(port) => {
                        let port = port.parse::<u16>()?;
                        AppConfig::Port(port)
                    }
                    None => Err(anyhow!("Port number not provided"))?,
                },
                "--replicaof" => {
                    let host = match args.next() {
                        Some(host) => host,
                        None => Err(anyhow!("replicaof host not provided"))?,
                    };
                    let port = match args.next() {
                        Some(port) => port,
                        None => Err(anyhow!("replicaof port number not provided"))?,
                    };
                    AppConfig::ReplicaOf(host, port)
                }
                _ => Err(anyhow!("Unknown argument"))?,
            };
            map.insert(arg, cli_arg);
        }

        // Setup matser replication id and offset
        match map.get("--replicaof") {
            None => {
                let alpha_numeric = b"abcdefghijklmnopqrstuvwxyz0123456789";
                let hash = (0..40)
                    .map(|_| {
                        let idx = rand::random::<usize>() % alpha_numeric.len();
                        *alpha_numeric.get(idx).unwrap()
                    })
                    .collect::<Vec<u8>>();
                let hash = String::from_utf8(hash).unwrap();
                map.insert("$$master_replid".to_string(), AppConfig::MasterReplId(hash));
                map.insert(
                    "$$master_repl_offset".to_string(),
                    AppConfig::MasterReplOffset(0),
                );
            }
            _ => {}
        }
        debug!("AppConfigs ➡️  {map:?}");
        Ok(map)
    }
}