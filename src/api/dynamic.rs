//! Dynamic feed API types and functions

use serde::Deserialize;

/// Dynamic feed response
#[derive(Debug, Deserialize)]
pub struct DynamicFeedData {
    pub items: Option<Vec<DynamicItem>>,
    pub offset: Option<String>,
    pub has_more: Option<bool>,
    pub update_num: Option<i32>,
}

/// Individual dynamic item
#[derive(Debug, Clone, Deserialize)]
pub struct DynamicItem {
    pub id_str: Option<String>,
    #[serde(rename = "type")]
    pub dynamic_type: Option<String>,
    pub modules: Option<DynamicModules>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DynamicModules {
    pub module_author: Option<ModuleAuthor>,
    pub module_dynamic: Option<ModuleDynamic>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModuleAuthor {
    pub name: Option<String>,
    pub face: Option<String>,
    pub mid: Option<i64>,
    pub pub_time: Option<String>,
    pub pub_ts: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModuleDynamic {
    pub major: Option<DynamicMajor>,
    pub desc: Option<DynamicDesc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DynamicMajor {
    #[serde(rename = "type")]
    pub major_type: Option<String>,
    pub archive: Option<ArchiveInfo>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ArchiveInfo {
    pub bvid: Option<String>,
    pub title: Option<String>,
    pub cover: Option<String>,
    pub desc: Option<String>,
    pub duration_text: Option<String>,
    pub stat: Option<ArchiveStat>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ArchiveStat {
    pub play: Option<String>,
    pub danmaku: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DynamicDesc {
    pub text: Option<String>,
}

impl DynamicItem {
    pub fn is_video(&self) -> bool {
        self.modules
            .as_ref()
            .and_then(|m| m.module_dynamic.as_ref())
            .and_then(|d| d.major.as_ref())
            .and_then(|m| m.major_type.as_ref())
            .map(|t| t == "MAJOR_TYPE_ARCHIVE")
            .unwrap_or(false)
    }

    pub fn author_name(&self) -> &str {
        self.modules
            .as_ref()
            .and_then(|m| m.module_author.as_ref())
            .and_then(|a| a.name.as_deref())
            .unwrap_or("未知")
    }

    pub fn author_face(&self) -> Option<&str> {
        self.modules
            .as_ref()
            .and_then(|m| m.module_author.as_ref())
            .and_then(|a| a.face.as_deref())
    }

    pub fn pub_time(&self) -> &str {
        self.modules
            .as_ref()
            .and_then(|m| m.module_author.as_ref())
            .and_then(|a| a.pub_time.as_deref())
            .unwrap_or("")
    }

    pub fn video_title(&self) -> Option<&str> {
        self.modules
            .as_ref()
            .and_then(|m| m.module_dynamic.as_ref())
            .and_then(|d| d.major.as_ref())
            .and_then(|m| m.archive.as_ref())
            .and_then(|a| a.title.as_deref())
    }

    pub fn video_cover(&self) -> Option<&str> {
        self.modules
            .as_ref()
            .and_then(|m| m.module_dynamic.as_ref())
            .and_then(|d| d.major.as_ref())
            .and_then(|m| m.archive.as_ref())
            .and_then(|a| a.cover.as_deref())
    }

    pub fn video_bvid(&self) -> Option<&str> {
        self.modules
            .as_ref()
            .and_then(|m| m.module_dynamic.as_ref())
            .and_then(|d| d.major.as_ref())
            .and_then(|m| m.archive.as_ref())
            .and_then(|a| a.bvid.as_deref())
    }

    pub fn video_play(&self) -> &str {
        self.modules
            .as_ref()
            .and_then(|m| m.module_dynamic.as_ref())
            .and_then(|d| d.major.as_ref())
            .and_then(|m| m.archive.as_ref())
            .and_then(|a| a.stat.as_ref())
            .and_then(|s| s.play.as_deref())
            .unwrap_or("-")
    }

    pub fn video_danmaku(&self) -> &str {
        self.modules
            .as_ref()
            .and_then(|m| m.module_dynamic.as_ref())
            .and_then(|d| d.major.as_ref())
            .and_then(|m| m.archive.as_ref())
            .and_then(|a| a.stat.as_ref())
            .and_then(|s| s.danmaku.as_deref())
            .unwrap_or("-")
    }

    pub fn video_duration(&self) -> &str {
        self.modules
            .as_ref()
            .and_then(|m| m.module_dynamic.as_ref())
            .and_then(|d| d.major.as_ref())
            .and_then(|m| m.archive.as_ref())
            .and_then(|a| a.duration_text.as_deref())
            .unwrap_or("")
    }
}
