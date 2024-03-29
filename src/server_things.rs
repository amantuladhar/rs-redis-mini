use anyhow::{anyhow, Context};
use std::{
    collections::HashMap,
    io::Write,
    net::TcpStream,
    sync::{Arc, RwLock},
    time::Duration,
};

use crate::{
    cli_args::CliArgs,
    command::Command,
    resp_parser::DataType,
    store::{Store, KEY_IS_MASTER, KEY_MASTER_REPL_OFFSET},
};
use crate::{store::KEY_MASTER_REPLID, LINE_ENDING};

pub fn parse_tcp_stream(
    mut stream: TcpStream,
    shared_map: Arc<RwLock<Store>>,
    cmd_args: Arc<HashMap<String, CliArgs>>,
) -> anyhow::Result<()> {
    loop {
        {
            // Test to check message format
            // use std::io::Read;
            // let mut buf = [0; 256];
            // stream.read(&mut buf)?;
            // println!("Content: {:?}", std::str::from_utf8(&buf).unwrap());
        }
        let mut reader: std::io::BufReader<&TcpStream> = std::io::BufReader::new(&stream);
        let command = Command::parse_with_reader(&mut reader)?;
        let msg = match &command {
            Command::Ping(_) => DataType::SimpleString("PONG".to_string()),
            Command::Echo(value) => DataType::SimpleString(value.clone()),
            Command::Set(key, value, do_get, exp_time) => {
                process_set_cmd(&shared_map, key, value, do_get, exp_time)?
            }
            Command::Get(key) => process_get_cmd(&shared_map, key)?,
            Command::Info(_) => process_info_cmd(&shared_map, &cmd_args),
            Command::ReplConf(_, _) => DataType::SimpleString("OK".to_string()),
            Command::PSync(_, _) => process_psync_cmd(&shared_map)?,
            Command::Noop => {
                // Do nothing
                break;
            }
            #[allow(unreachable_patterns)]
            _ => Err(anyhow!("Unknown command"))?,
        }
        .to_string();
        println!("🙏 >>> Response: {:?} <<<", msg);
        stream
            .write_all(msg.as_bytes())
            .context("Unable to write to TcpStream")?;
        do_follow_up_if_needed(&command, &stream)?;
    }
    Ok(())
}

fn do_follow_up_if_needed(command: &Command, mut stream: &TcpStream) -> anyhow::Result<()> {
    match command {
        Command::PSync(_, _) => {
            pub fn decode_hex(s: &str) -> anyhow::Result<Vec<u8>> {
                let r: Vec<u8> = (0..s.len())
                    .step_by(2)
                    .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
                    .collect();
                Ok(r)
            }
            let hex = "524544495330303131fa0972656469732d76657205372e322e30fa0a72656469732d62697473c040fa056374696d65c26d08bc65fa08757365642d6d656dc2b0c41000fa08616f662d62617365c000fff06e3bfec0ff5aa2";
            // let bytes = String::from_utf8(decode_hex(hex)?)?;
            // println!("Bytes: {:?}", bytes);
            // let body = std::str::from_utf8(bytes)?;
            let msg = DataType::NotBulkString(format!("{}", hex).into()).to_string();
            stream.write_all(msg.as_bytes())?;
        }
        _ => {}
    };
    Ok(())
}

fn process_psync_cmd(shared_map: &Arc<RwLock<Store>>) -> anyhow::Result<DataType> {
    let mut map = shared_map.write().unwrap();
    map.get(KEY_IS_MASTER.into())
        .ok_or_else(|| anyhow!("Not a master"))?;
    let master_replid = map
        .get(KEY_MASTER_REPLID.into())
        .ok_or_else(|| anyhow!("No master replid found"))?;
    let master_repl_offset = map.get(KEY_MASTER_REPL_OFFSET.into()).unwrap_or("0".into());
    Ok(DataType::SimpleString(format!(
        "FULLRESYNC {} {}",
        master_replid, master_repl_offset
    )))
}

fn process_info_cmd(
    shared_map: &Arc<RwLock<Store>>,
    cmd_args: &Arc<HashMap<String, CliArgs>>,
) -> DataType {
    let is_replica = cmd_args.get("--replicaof").is_some();
    let mut msg = vec![
        format!("# Replication"),
        format!("role:{}", if is_replica { "slave" } else { "master" }),
    ];
    if !is_replica {
        let mut map = shared_map.write().unwrap();
        let master_replid = map.get(KEY_MASTER_REPLID.into()).unwrap();
        let master_repl_offset = map.get(KEY_MASTER_REPL_OFFSET.into()).unwrap_or("0".into());
        msg.push(format!("master_replid:{}", master_replid));
        msg.push(format!("master_repl_offset:{}", master_repl_offset))
    }
    DataType::BulkString(format!("{}{LINE_ENDING}", msg.join(LINE_ENDING)))
}

fn process_get_cmd(shared_map: &Arc<RwLock<Store>>, key: &String) -> anyhow::Result<DataType> {
    // Write lock here because get for now also removes expired keys
    let mut map = shared_map.write().unwrap();
    let msg = match map.get(key.clone()) {
        Some(value) => DataType::BulkString(value.to_string()),
        None => DataType::NullBulkString,
    };
    Ok(msg)
}

fn process_set_cmd(
    shared_map: &Arc<RwLock<Store>>,
    key: &String,
    value: &String,
    do_get: &bool,
    exp_time: &Option<Duration>,
) -> anyhow::Result<DataType> {
    let mut map = shared_map.write().unwrap();
    let old_value = map.get(key.clone());
    map.set(key.clone(), value.clone(), exp_time.clone());
    let msg = match do_get {
        true => match old_value {
            Some(old_value) => DataType::BulkString(old_value),
            None => DataType::NullBulkString,
        },
        false => DataType::SimpleString("OK".to_string()),
    };
    Ok(msg)
}
