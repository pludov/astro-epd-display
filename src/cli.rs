use clap::Parser;

#[derive(clap::ValueEnum, Default, Clone, Debug)]
pub enum Driver {
    Epd,
    #[default]
    Stdout,
}

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(short, long, default_value = "stdout")]
    pub driver: Driver,

    #[arg(long, default_value = "128")]
    pub width: u32,
    #[arg(long, default_value = "64")]
    pub height: u32,
}
