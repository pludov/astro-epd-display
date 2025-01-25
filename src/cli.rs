use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug, Clone)]
pub struct EpdConfig {
    #[arg(
        long,
        default_value = "6",
        help = "Max number of refresh per pixel before a full upgrade is triggered"
    )]
    pub max_partial_per_pixel: u8,
}

#[derive(Subcommand, Default, Clone, Debug)]
pub enum Driver {
    Epd(EpdConfig),
    #[default]
    Stdout,
}

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub driver: Option<Driver>,

    #[arg(short, long, help = "Path to template")]
    pub template: Option<PathBuf>,

    #[arg(short, long, default_value = "3000", help = "Port for http server")]
    pub port: u16,

    #[arg(long, default_value = "128")]
    pub width: u32,
    #[arg(long, default_value = "64")]
    pub height: u32,
}
