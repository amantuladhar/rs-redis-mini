#![warn(clippy::all)]

use tracing::debug;

use crate::{
    database::Database, log::setup_log, rds_file::parse_rdb_file, replication::ReplicationEvent,
    server::Server, slave::Slave,
};

pub(crate) mod app_config;
pub(crate) mod cmd_parser;
pub(crate) mod cmd_processor;
pub(crate) mod database;
pub(crate) mod log;
pub(crate) mod rds_file;
pub(crate) mod replication;
pub(crate) mod resp_type;
pub(crate) mod server;
pub(crate) mod slave;

pub const LINE_ENDING: &str = "\r\n";
pub const NEW_LINE: u8 = b'\n';

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    setup_log()?;
    debug!("🚀🚀🚀 Logs from your program will appear here! 🚀🚀🚀");
    Database::new();
    parse_rdb_file().await?;
    ReplicationEvent::setup();
    Slave::setup().await?;
    Server::start().await?;

    Ok(())
}
