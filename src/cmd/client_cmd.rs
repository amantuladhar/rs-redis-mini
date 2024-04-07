use anyhow::bail;

use crate::{fdbg, resp_type::RESPType};

pub enum ClientCmd {
    Ping,
    Echo(String),
    Get { key: String },
    Set { key: String, value: String },
    CustomNewLine,
    EOF,
}

impl ClientCmd {
    pub fn from_resp_type(resp_type: &RESPType) -> anyhow::Result<Self> {
        match resp_type {
            RESPType::Array(items) => parse_client_cmd(&items),
            RESPType::CustomNewLine => Ok(ClientCmd::CustomNewLine),
            RESPType::EOF => Ok(ClientCmd::EOF),
            _ => bail!("Client command must be of type array"),
        }
    }
}

fn parse_client_cmd(items: &[RESPType]) -> anyhow::Result<ClientCmd> {
    if items.is_empty() {
        bail!("Client command array must have at least one element");
    }
    let Some(RESPType::BulkString(cmd)) = items.get(0) else {
        bail!("First element of client command array must be a bulk string");
    };
    let cmd = cmd.to_uppercase();
    match cmd.as_str() {
        "PING" => Ok(ClientCmd::Ping),
        "ECHO" => parse_echo_cmd(&items[1..]),
        "SET" => parse_set_cmd(&items[1..]),
        "GET" => parse_get_cmd(&items[1..]),
        _ => bail!("Unknown client command: {}", cmd),
    }
}
fn parse_get_cmd(items: &[RESPType]) -> anyhow::Result<ClientCmd> {
    let Some(RESPType::BulkString(key)) = items.get(0) else {
        bail!(fdbg!("GEt command must have at least key"));
    };
    Ok(ClientCmd::Get {
        key: key.to_owned(),
    })
}

fn parse_set_cmd(items: &[RESPType]) -> anyhow::Result<ClientCmd> {
    let Some(RESPType::BulkString(key)) = items.get(0) else {
        bail!(fdbg!("SET command must have at least key"));
    };
    let Some(RESPType::BulkString(value)) = items.get(1) else {
        bail!(fdbg!("SET command must have at least value"));
    };
    Ok(ClientCmd::Set {
        key: key.to_owned(),
        value: value.to_owned(),
    })
}
fn parse_echo_cmd(items: &[RESPType]) -> anyhow::Result<ClientCmd> {
    let Some(RESPType::BulkString(value)) = items.get(0) else {
        bail!(fdbg!("ECHO command must have at least one argument"));
    };
    Ok(ClientCmd::Echo(value.to_owned()))
}
