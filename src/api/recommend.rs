//! Video recommendation API types

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct RecommendData {
    pub item: Vec<VideoItem>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VideoItem {
    pub id: i64,
    pub bvid: Option<String>,
    pub cid: Option<i64>,
    pub goto: String,
    pub uri: Option<String>,
    pub pic: Option<String>,
    pub title: Option<String>,
    pub duration: Option<i64>,
    pub pubdate: Option<i64>,
    pub owner: Option<VideoOwner>,
    pub stat: Option<VideoStat>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VideoOwner {
    pub mid: i64,
    pub name: String,
    pub face: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VideoStat {
    pub view: Option<i64>,
    pub like: Option<i64>,
    pub danmaku: Option<i64>,
}

impl VideoItem {
    /// Format duration as mm:ss
    pub fn format_duration(&self) -> String {
        if let Some(duration) = self.duration {
            let minutes = duration / 60;
            let seconds = duration % 60;
            format!("{:02}:{:02}", minutes, seconds)
        } else {
            "--:--".to_string()
        }
    }

    /// Format view count (e.g., 1.2万)
    pub fn format_views(&self) -> String {
        if let Some(stat) = &self.stat {
            if let Some(view) = stat.view {
                if view >= 10000 {
                    format!("{:.1}万", view as f64 / 10000.0)
                } else {
                    view.to_string()
                }
            } else {
                "-".to_string()
            }
        } else {
            "-".to_string()
        }
    }

    /// Get author name
    pub fn author_name(&self) -> &str {
        self.owner.as_ref().map(|o| o.name.as_str()).unwrap_or("-")
    }
}
