use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(name = "nrelay")]
#[command(about = "NRelay reverse tunnel server")]
pub struct Config {
    #[arg(long, default_value = "0.0.0.0")]
    pub bind_addr: String,

    #[arg(long, default_value = "7000")]
    pub client_port: u16,

    #[arg(long, default_value = "7001")]
    pub admin_port: u16,

    #[arg(long, env = "NRELAY_ADMIN_TOKEN")]
    pub admin_token: Option<String>,

    #[arg(long, default_value = "example.com")]
    pub public_domain: String,

    #[arg(long)]
    pub relay_domain: Option<String>,
}

impl Config {
    pub fn admin_token(&self) -> anyhow::Result<String> {
        self.admin_token
            .clone()
            .ok_or_else(|| anyhow::anyhow!(
                "Admin token not provided. Set --admin-token, NRELAY_ADMIN_TOKEN env var, or .env file"
            ))
    }
}
