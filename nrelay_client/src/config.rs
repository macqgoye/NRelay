use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(name = "nrelay-client")]
#[command(about = "NRelay tunnel client")]
pub struct Config {
    #[arg(long, default_value = "127.0.0.1:7000")]
    pub server_addr: String,

    #[arg(long)]
    pub tunnel_token: String,

    #[arg(long, default_value = "127.0.0.1")]
    pub local_addr: String,

    #[arg(long)]
    pub local_port: u16,
}
