use crate::cli::{OriginKind, TunnelProtocol};
use crate::config;
use anyhow::{Context, Result};
use colored::Colorize;
use nrelay_core::TunnelKind;
use std::process::{Command, Stdio};

pub async fn start_tunnel(
    protocol: TunnelProtocol,
    target: &str,
    origin_id: Option<&str>,
) -> Result<()> {
    let origin_id = match origin_id {
        Some(id) => id.to_string(),
        None => config::get_default_origin()?
            .context("No default origin set. Use 'nrelay origin use <id>' or specify --origin")?,
    };

    let origin = config::load_origin(&origin_id)?;

    println!(
        "{} Starting {} tunnel...",
        "→".cyan(),
        protocol.as_str().yellow()
    );
    println!("  Origin: {}", origin_id.cyan());
    println!("  Target: {}", target.yellow());

    let (local_addr, local_port) = parse_target(target)?;

    let tunnel_info = match origin.kind {
        OriginKind::Server => create_tunnel_server(&origin, protocol, local_port).await?,
        OriginKind::Service => create_tunnel_service(&origin, protocol, local_port).await?,
    };

    println!();
    println!("{}", "Tunnel created successfully!".green().bold());
    println!();
    println!("  {} {}", "Tunnel ID:".bold(), tunnel_info.tunnel_id);
    println!("  {} {}", "Protocol:".bold(), protocol.as_str());

    if let Some(hostname) = &tunnel_info.public_hostname {
        println!("  {} {}", "Public URL:".bold(), hostname.cyan());
    }

    if let Some(port) = tunnel_info.public_port {
        println!("  {} {}", "Public Port:".bold(), port.to_string().cyan());
    }

    println!();
    println!("{} Launching client...", "→".cyan());

    let client_exe = get_client_executable()?;

    // Use relay_addr and relay_port from tunnel_info if available
    // Otherwise fall back to extracting from origin URL
    let server_addr = if let Some(relay_addr) = &tunnel_info.relay_addr {
        // If relay_addr is set, use it with relay_port
        let port = tunnel_info.relay_port.unwrap_or(7000);
        format!("{}:{}", relay_addr, port)
    } else {
        // If no relay_addr, use origin URL but replace port with relay_port
        let origin_addr = extract_server_addr(&origin.url)?;
        if let Some(relay_port) = tunnel_info.relay_port {
            // Replace port in origin_addr
            if let Some(colon_pos) = origin_addr.rfind(':') {
                format!("{}:{}", &origin_addr[..colon_pos], relay_port)
            } else {
                format!("{}:{}", origin_addr, relay_port)
            }
        } else {
            origin_addr
        }
    };

    let mut child = Command::new(&client_exe)
        .arg("--server-addr")
        .arg(&server_addr)
        .arg("--tunnel-token")
        .arg(&tunnel_info.access_token)
        .arg("--local-addr")
        .arg(&local_addr)
        .arg("--local-port")
        .arg(local_port.to_string())
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .context(format!("Failed to execute nrelay-client: {}", client_exe))?;

    println!();
    println!("{}", "Client is running. Press Ctrl+C to stop.".green());
    println!();

    let status = child.wait().context("Failed to wait for client process")?;

    if !status.success() {
        anyhow::bail!("Client exited with error: {:?}", status.code());
    }

    Ok(())
}

#[derive(Debug, serde::Deserialize)]
struct TunnelInfo {
    tunnel_id: String,
    access_token: String,
    public_hostname: Option<String>,
    public_port: Option<u16>,
    relay_addr: Option<String>,
    relay_port: Option<u16>,
}

async fn create_tunnel_server(
    origin: &config::Origin,
    protocol: TunnelProtocol,
    local_port: u16,
) -> Result<TunnelInfo> {
    let kind = match protocol {
        TunnelProtocol::TcpRaw => TunnelKind::TcpRaw,
        TunnelProtocol::UdpRaw => TunnelKind::UdpRaw,
        TunnelProtocol::Http => TunnelKind::Http,
        TunnelProtocol::Https => TunnelKind::Https,
        TunnelProtocol::Minecraft => TunnelKind::Minecraft,
        TunnelProtocol::TlsSni => TunnelKind::TlsSni,
    };

    let create_url = format!("{}/tunnels", origin.url.trim_end_matches('/'));

    let payload = serde_json::json!({
        "kind": kind,
        "local_port": local_port,
        "fixed_public_port": None::<u16>,
        "hostname": None::<String>,
    });

    let client = reqwest::Client::new();
    let response = client
        .post(&create_url)
        .header("Authorization", format!("Bearer {}", origin.token))
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await
        .context("Failed to send tunnel creation request")?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        anyhow::bail!(
            "Failed to create tunnel. Status: {}, Body: {}",
            status,
            body
        );
    }

    let tunnel_info: TunnelInfo = response
        .json()
        .await
        .context("Failed to parse tunnel creation response")?;

    Ok(tunnel_info)
}

async fn create_tunnel_service(
    _origin: &config::Origin,
    _protocol: TunnelProtocol,
    _local_port: u16,
) -> Result<TunnelInfo> {
    anyhow::bail!("Service origins are not yet implemented. Please use a server origin.")
}

fn parse_target(target: &str) -> Result<(String, u16)> {
    if target.contains(':') {
        let parts: Vec<&str> = target.split(':').collect();
        if parts.len() != 2 {
            anyhow::bail!("Invalid target format. Use 'port' or 'addr:port'");
        }

        let addr = parts[0].to_string();
        let port: u16 = parts[1].parse().context("Invalid port number")?;

        Ok((addr, port))
    } else {
        let port: u16 = target.parse().context("Invalid port number")?;

        Ok(("127.0.0.1".to_string(), port))
    }
}

fn extract_server_addr(url: &str) -> Result<String> {
    let url = url.trim_end_matches('/');

    let without_scheme = url
        .strip_prefix("http://")
        .or_else(|| url.strip_prefix("https://"))
        .unwrap_or(url);

    let addr = without_scheme.split('/').next().unwrap_or(without_scheme);

    if addr.contains(':') {
        Ok(addr.to_string())
    } else {
        Ok(format!("{}:7000", addr))
    }
}

fn get_client_executable() -> Result<String> {
    #[cfg(target_os = "windows")]
    let exe = "nrelay-client.exe";

    #[cfg(not(target_os = "windows"))]
    let exe = "nrelay-client";

    let current_dir = std::env::current_exe()
        .context("Failed to get current executable path")?
        .parent()
        .context("Failed to get parent directory")?
        .to_path_buf();

    let client_path = current_dir.join(exe);

    if client_path.exists() {
        return Ok(client_path.to_string_lossy().to_string());
    }

    if let Ok(path) = which::which(exe) {
        return Ok(path.to_string_lossy().to_string());
    }

    Ok(exe.to_string())
}
