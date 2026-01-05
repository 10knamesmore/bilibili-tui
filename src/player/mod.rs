//! mpv player integration

use crate::storage::Credentials;
use anyhow::Result;
use std::process::Stdio;
use tokio::process::Command;

/// Play a video using mpv with yt-dlp
pub async fn play_video(bvid: &str, credentials: Option<&Credentials>) -> Result<()> {
    let video_url = format!("https://www.bilibili.com/video/{}", bvid);

    let mut cmd = Command::new("mpv");

    // Redirect stdout/stderr to null to prevent interfering with TUI
    cmd.stdout(Stdio::null());
    cmd.stderr(Stdio::null());

    let mut cookie_path_to_clean = None;

    // If we have credentials, export cookies for yt-dlp
    if let Some(creds) = credentials {
        let cookie_path = crate::storage::export_cookies_for_ytdlp(creds)?;
        cmd.arg(format!(
            "--ytdl-raw-options=cookies={}",
            cookie_path.display()
        ));
        cookie_path_to_clean = Some(cookie_path);
    }

    cmd.arg("--force-window=immediate");
    cmd.arg(&video_url);

    // Spawn mpv process
    let mut child = cmd.spawn()?;

    // Wait for mpv to exit asynchronously
    let _ = child.wait().await?;

    // Clean up cookie file
    if let Some(path) = cookie_path_to_clean {
        let _ = tokio::fs::remove_file(path).await;
    }

    Ok(())
}
