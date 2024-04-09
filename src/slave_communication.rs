use std::collections::HashMap;

use tokio::{
    io::{AsyncWriteExt, BufReader},
    net::TcpStream,
    sync::{mpsc, oneshot},
};
use tracing::debug;

use crate::resp_type::{parser::parse_request, RESPType};

pub async fn setup_master_to_slave_communication() -> mpsc::Sender<MasterToSlaveCmd> {
    use MasterToSlaveCmd::*;
    let (tx, mut rx) = mpsc::channel::<MasterToSlaveCmd>(5);
    tokio::spawn(async move {
        let mut streams_map: HashMap<String, TcpStream> = HashMap::new();
        while let Some(cmd) = rx.recv().await {
            match cmd {
                SaveStream { host, port, stream } => {
                    let key = format!("{host}:{port}");
                    streams_map.insert(key, stream);
                }
                Set {
                    key,
                    value,
                    flags: _flags,
                } => {
                    let msg = RESPType::Array(vec![
                        RESPType::BulkString("SET".to_string()),
                        RESPType::BulkString(key.clone()),
                        RESPType::BulkString(value.clone()),
                    ]);
                    for (_, v) in &mut streams_map {
                        let _ = v.write_all(&msg.as_bytes()).await;
                        v.flush().await.unwrap();
                    }
                }
                GetNumOfReplicas { resp: recv_chan } => {
                    let _ = recv_chan.send(streams_map.len());
                }
                GetAck { min_ack, resp } => {
                    let mut acks_received = 0;

                    let req = RESPType::Array(vec![
                        RESPType::BulkString("REPLCONF".to_string()),
                        RESPType::BulkString("GETACK".to_string()),
                        RESPType::BulkString("*".to_string()),
                    ]);
                    for (_, streams) in &mut streams_map {
                        let (reader, mut writer) = streams.split();
                        debug!("Sending GET ACK TO slave");
                        let _ = writer.write_all(&req.as_bytes()).await;
                        // streams.flush().await.unwrap();
                        debug!("Writing to one slave");
                        let span = tracing::span!(tracing::Level::DEBUG, "READING ACK FROM CLIENT");
                        let _guard = span.enter();
                        debug!("Creating bufferred reader");
                        let mut reader = BufReader::new(reader);
                        let resp_type = parse_request(&mut reader).await.unwrap();
                        debug!("RESP from slave - {:?}", resp_type);
                        acks_received += 1;
                        debug!(
                            "Acks received {:?} -- min_acks -- {}",
                            acks_received, min_ack
                        );
                        if acks_received >= min_ack {
                            break;
                        }
                    }
                    debug!("Respoding to the onshot channel");
                    let _ = resp.send(acks_received);
                }
            }
        }
    });
    tx
}

#[derive(Debug)]
pub enum MasterToSlaveCmd {
    SaveStream {
        host: String,
        port: u16,
        stream: TcpStream,
    },
    Set {
        key: String,
        value: String,
        flags: HashMap<String, String>,
    },
    GetNumOfReplicas {
        resp: oneshot::Sender<usize>,
    },
    GetAck {
        min_ack: usize,
        resp: oneshot::Sender<usize>,
    },
}