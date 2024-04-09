use std::collections::HashMap;

use anyhow::bail;

use super::client_cmd::ClientCmd;

#[derive(Debug)]
pub enum SlaveCmd {
    Ping,
    Set {
        key: String,
        value: String,
        flags: HashMap<String, String>,
    },
    ReplConf {
        key: String,
        value: String,
    },
}
impl SlaveCmd {
    // Hack to convert ClientCmd to SlaveCmd
    // Probably should refactor this to use a trait
    pub fn from_client_cmd(client_cmd: &ClientCmd) -> anyhow::Result<Self> {
        match client_cmd {
            ClientCmd::Ping => Ok(SlaveCmd::Ping),
            ClientCmd::Set { key, value, flags } => Ok(SlaveCmd::Set {
                key: key.clone(),
                value: value.clone(),
                flags: flags.clone(),
            }),
            ClientCmd::ReplConf { key, value } => Ok(SlaveCmd::ReplConf {
                key: key.clone(),
                value: value.clone(),
            }),
            _ => bail!("Only SET command is supported for now = {:?}", client_cmd),
        }
    }
}
