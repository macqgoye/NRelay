use crate::cli::OriginKind;
use crate::config::{self, Origin};
use anyhow::Result;
use colored::Colorize;
use tabled::{Table, Tabled};

#[derive(Tabled)]
struct OriginRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "Kind")]
    kind: String,
    #[tabled(rename = "URL")]
    url: String,
    #[tabled(rename = "Default")]
    default: String,
}

pub async fn list_origins() -> Result<()> {
    let origin_ids = config::list_origins()?;
    let default_origin = config::get_default_origin()?;

    if origin_ids.is_empty() {
        println!("{}", "No origins configured.".yellow());
        println!("\nAdd an origin with: {} origin add <id> --url <url> --token <token>", "nrelay".cyan());
        return Ok(());
    }

    let mut rows = Vec::new();

    for id in origin_ids {
        let origin = config::load_origin(&id)?;
        let is_default = default_origin.as_ref() == Some(&id);

        rows.push(OriginRow {
            id: id.clone(),
            kind: format!("{}", origin.kind),
            url: origin.url.clone(),
            default: if is_default {
                "✓".green().to_string()
            } else {
                "".to_string()
            },
        });
    }

    let table = Table::new(rows).to_string();
    println!("\n{}", table);
    println!();

    Ok(())
}

pub async fn add_origin(id: &str, url: &str, token: &str, kind: OriginKind) -> Result<()> {
    let origin = Origin {
        id: id.to_string(),
        kind,
        url: url.to_string(),
        token: token.to_string(),
    };

    config::save_origin(&origin)?;

    println!("{} Origin '{}' added successfully", "✓".green(), id.cyan());

    let default = config::get_default_origin()?;
    if default.is_none() {
        config::set_default_origin(id)?;
        println!("{} Set as default origin", "✓".green());
    }

    Ok(())
}

pub async fn set_origin(id: &str, key: &str, value: &str) -> Result<()> {
    let mut origin = config::load_origin(id)?;

    match key {
        "url" => origin.url = value.to_string(),
        "token" => origin.token = value.to_string(),
        "kind" => {
            origin.kind = value
                .parse()
                .map_err(|e: String| anyhow::anyhow!(e))?;
        }
        _ => anyhow::bail!("Unknown key '{}'. Valid keys: url, token, kind", key),
    }

    config::save_origin(&origin)?;

    println!(
        "{} Origin '{}' updated: {} = {}",
        "✓".green(),
        id.cyan(),
        key.yellow(),
        if key == "token" {
            "***".to_string()
        } else {
            value.to_string()
        }
    );

    Ok(())
}

pub async fn get_origin(id: &str, key: &str) -> Result<()> {
    let origin = config::load_origin(id)?;

    let value = match key {
        "url" => origin.url,
        "token" => origin.token,
        "kind" => format!("{}", origin.kind),
        _ => anyhow::bail!("Unknown key '{}'. Valid keys: url, token, kind", key),
    };

    println!("{}", value);

    Ok(())
}

pub async fn remove_origin(id: &str) -> Result<()> {
    config::delete_origin(id)?;

    let default = config::get_default_origin()?;
    if default.as_ref() == Some(&id.to_string()) {
        let mut cfg = config::load_config()?;
        cfg.default_origin = None;
        config::save_config(&cfg)?;
    }

    println!("{} Origin '{}' removed", "✓".green(), id.cyan());

    Ok(())
}

pub async fn use_origin(id: &str) -> Result<()> {
    config::load_origin(id)?;
    config::set_default_origin(id)?;

    println!("{} Default origin set to '{}'", "✓".green(), id.cyan());

    Ok(())
}