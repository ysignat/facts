#![allow(clippy::struct_field_names)]
use std::net::{IpAddr, Ipv4Addr};

use clap::{value_parser, Args, Parser, ValueEnum};
use tracing::Level;

#[derive(Parser, Debug)]
#[clap(version, about)]
pub struct Config {
    #[command(flatten)]
    pub runtime: Runtime,
    #[command(flatten)]
    pub logging: Logging,
    #[command(flatten)]
    pub storage: Storage,
    #[command(flatten)]
    pub authentication: Authentication,
}

#[derive(Args, Clone, Debug)]
pub struct Runtime {
    #[arg(long, env = "HOST", default_value = Ipv4Addr::LOCALHOST.to_string())]
    pub bind_host: IpAddr,
    #[arg(long, env = "PORT", value_parser = value_parser!(u16).range(1..), default_value = "8080")]
    pub bind_port: u16,
}

#[derive(Args, Clone, Debug)]
pub struct Logging {
    #[arg(long, env, default_value = "INFO")]
    pub log_level: Level,
    #[arg(long, env, default_value_t, value_enum)]
    pub log_format: LogFormat,
}

#[derive(Clone, ValueEnum, Default, Debug)]
pub enum LogFormat {
    Json,
    #[default]
    Default,
    Pretty,
}

#[derive(Clone, ValueEnum, Default, Debug)]
pub enum StorageType {
    Mocked,
    #[default]
    Sqlx,
}

#[derive(Args, Clone, Debug)]
pub struct Storage {
    #[arg(long, env, default_value_t, value_enum)]
    pub storage_type: StorageType,
    #[arg(long, env, default_value = String::new(), value_enum)]
    pub storage_dsn: String,
}

#[derive(Args, Clone, Debug)]
pub struct Authentication {
    #[arg(long, env)]
    pub password_hash: String,
}
