use std::collections::HashMap;

use anyhow::bail;

use super::server_command::ServerCommand;

/// These are commands that are sent by the master to the slave
#[derive(Debug)]
pub enum SlaveCommand {
    Ping,
    Set {
        key: String,
        value: String,
        flags: HashMap<String, String>,
    },
    ReplConf {
        #[allow(dead_code)]
        key: String,
        #[allow(dead_code)]
        value: String,
    },
}

impl SlaveCommand {
    // Need to find a better way later
    // Hack for now, because RESPType can be converted to ServerCommand
    pub fn from(client_cmd: &ServerCommand) -> anyhow::Result<Self> {
        match client_cmd {
            ServerCommand::Ping => Ok(SlaveCommand::Ping),
            ServerCommand::Set { key, value, flags } => Ok(SlaveCommand::Set {
                key: key.clone(),
                value: value.to_owned(),
                flags: flags.clone(),
            }),
            ServerCommand::ReplConf { key, value } => Ok(SlaveCommand::ReplConf {
                key: key.clone(),
                value: value.clone(),
            }),
            _ => bail!(
                "Only PING,SET and REPLCONF command is supported for now = {:?}",
                client_cmd
            ),
        }
    }
}
