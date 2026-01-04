//! Video info API types

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct VideoInfo {
    pub bvid: String,
    pub aid: i64,
    pub cid: i64,
    pub title: String,
    pub desc: Option<String>,
    pub owner: VideoOwner,
    pub stat: VideoStat,
    pub pages: Option<Vec<VideoPage>>,
}

#[derive(Debug, Deserialize)]
pub struct VideoOwner {
    pub mid: i64,
    pub name: String,
    pub face: String,
}

#[derive(Debug, Deserialize)]
pub struct VideoStat {
    pub view: i64,
    pub danmaku: i64,
    pub like: i64,
    pub coin: i64,
    pub favorite: i64,
    pub share: i64,
}

#[derive(Debug, Deserialize)]
pub struct VideoPage {
    pub cid: i64,
    pub page: i32,
    pub part: String,
    pub duration: i64,
}
