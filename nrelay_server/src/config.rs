use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(name = "nrelay")]
#[command(about = "NRelay reverse tunnel server")]
pub struct Config {
    #[arg(long, default_value = "0.0.0.0")]
    pub relay_bind: String,

    #[arg(long, default_value = "7000")]
    pub relay_port: u16,

    #[arg(long, default_value = "0.0.0.0")]
    pub admin_bind: String,

    #[arg(long, default_value = "7001")]
    pub admin_port: u16,

    #[arg(long, env = "NRELAY_ADMIN_TOKEN")]
    pub admin_token: Option<String>,

    #[arg(long, default_value = "localhost")]
    pub relay_domain: String,
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
