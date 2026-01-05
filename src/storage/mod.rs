//! Credential storage and persistence

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// User credentials from Bilibili login
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    pub sessdata: String,
    pub bili_jct: String,
    pub dede_user_id: String,
    pub dede_user_id_ckmd5: Option<String>,
    pub refresh_token: Option<String>,
}

impl Credentials {
    pub fn from_cookies(
        cookies: &[(String, String)],
        refresh_token: Option<String>,
    ) -> Option<Self> {
        let mut sessdata = None;
        let mut bili_jct = None;
        let mut dede_user_id = None;
        let mut dede_user_id_ckmd5 = None;

        for (name, value) in cookies {
            match name.as_str() {
                "SESSDATA" => sessdata = Some(value.clone()),
                "bili_jct" => bili_jct = Some(value.clone()),
                "DedeUserID" => dede_user_id = Some(value.clone()),
                "DedeUserID__ckMd5" => dede_user_id_ckmd5 = Some(value.clone()),
                _ => {}
            }
        }

        Some(Credentials {
            sessdata: sessdata?,
            bili_jct: bili_jct?,
            dede_user_id: dede_user_id?,
            dede_user_id_ckmd5,
            refresh_token,
        })
    }
}

/// Get the config directory path
fn get_config_dir() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?
        .join("bilibili-tui");

    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)?;
    }

    Ok(config_dir)
}

/// Get the credentials file path
fn get_credentials_path() -> Result<PathBuf> {
    Ok(get_config_dir()?.join("credentials.json"))
}

/// Save credentials to disk
pub fn save_credentials(credentials: &Credentials) -> Result<()> {
    let path = get_credentials_path()?;
    let json = serde_json::to_string_pretty(credentials)?;
    fs::write(path, json)?;
    Ok(())
}

/// Load credentials from disk
pub fn load_credentials() -> Result<Credentials> {
    let path = get_credentials_path()?;
    let json = fs::read_to_string(path)?;
    let credentials: Credentials = serde_json::from_str(&json)?;
    Ok(credentials)
}

/// Export cookies in Netscape format for yt-dlp
pub fn export_cookies_for_ytdlp(credentials: &Credentials) -> Result<PathBuf> {
    let path = get_config_dir()?.join("cookies.txt");

    let content = format!(
        "# Netscape HTTP Cookie File\n\
        .bilibili.com\tTRUE\t/\tTRUE\t0\tSESSDATA\t{}\n\
        .bilibili.com\tTRUE\t/\tFALSE\t0\tbili_jct\t{}\n\
        .bilibili.com\tTRUE\t/\tFALSE\t0\tDedeUserID\t{}\n",
        credentials.sessdata, credentials.bili_jct, credentials.dede_user_id
    );

    fs::write(&path, content)?;
    Ok(path)
}
