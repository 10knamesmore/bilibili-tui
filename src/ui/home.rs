//! Homepage with video recommendations in a grid layout with cover images

use super::Component;
use crate::api::client::ApiClient;
use crate::api::recommend::VideoItem;
use crate::app::AppAction;
use image::DynamicImage;
use ratatui::{
    crossterm::event::KeyCode,
    prelude::*,
    widgets::*,
};
use ratatui_image::{picker::Picker, protocol::StatefulProtocol, StatefulImage};

/// Video card with cached cover image
pub struct VideoCard {
    pub video: VideoItem,
    pub cover: Option<StatefulProtocol>,
}

pub struct HomePage {
    videos: Vec<VideoCard>,
    selected_index: usize,
    loading: bool,
    error_message: Option<String>,
    scroll_row: usize,
    picker: Picker,
    columns: usize,
    card_height: u16,
    images_loaded: bool,
}

impl HomePage {
    pub fn new() -> Self {
        // Try to detect terminal graphics protocol (Kitty/Sixel/iTerm2)
        // Fall back to halfblocks if detection fails
        let picker = Picker::from_query_stdio().unwrap_or_else(|_| Picker::halfblocks());
        
        Self {
            videos: Vec::new(),
            selected_index: 0,
            loading: true,
            error_message: None,
            scroll_row: 0,
            picker,
            columns: 3,
            card_height: 12,
            images_loaded: false,
        }
    }

    pub async fn load_recommendations(&mut self, api_client: &ApiClient) {
        self.loading = true;
        self.error_message = None;
        self.images_loaded = false;

        match api_client.get_recommendations().await {
            Ok(videos) => {
                self.videos = videos
                    .into_iter()
                    .map(|video| VideoCard {
                        video,
                        cover: None,
                    })
                    .collect();
                self.loading = false;
                self.selected_index = 0;
                self.scroll_row = 0;
            }
            Err(e) => {
                self.error_message = Some(format!("加载推荐视频失败: {}", e));
                self.loading = false;
            }
        }
    }

    /// Load cover images for visible videos (call this in tick)
    pub async fn load_visible_covers(&mut self) {
        if self.images_loaded || self.videos.is_empty() {
            return;
        }

        // Load covers for current visible range
        let start = self.scroll_row * self.columns;
        let end = (start + self.columns * 3).min(self.videos.len());
        
        for idx in start..end {
            if self.videos[idx].cover.is_some() {
                continue;
            }
            
            if let Some(pic_url) = self.videos[idx].video.pic.clone() {
                // Download and process image
                if let Some(img) = Self::download_image(&pic_url).await {
                    self.videos[idx].cover = Some(self.picker.new_resize_protocol(img));
                }
            }
        }
        
        // Mark as loaded if all visible have covers
        let all_visible_loaded = (start..end).all(|i| {
            self.videos[i].cover.is_some() || self.videos[i].video.pic.is_none()
        });
        if all_visible_loaded {
            self.images_loaded = true;
        }
    }

    async fn download_image(url: &str) -> Option<DynamicImage> {
        let response = reqwest::get(url).await.ok()?;
        let bytes = response.bytes().await.ok()?;
        image::load_from_memory(&bytes).ok()
    }

    fn visible_rows(&self, height: u16) -> usize {
        let available_height = height.saturating_sub(5);
        (available_height / self.card_height).max(1) as usize
    }

    fn selected_row(&self) -> usize {
        self.selected_index / self.columns
    }

    fn update_scroll(&mut self, visible_rows: usize) {
        let current_row = self.selected_row();
        if current_row < self.scroll_row {
            self.scroll_row = current_row;
        } else if current_row >= self.scroll_row + visible_rows {
            self.scroll_row = current_row - visible_rows + 1;
        }
        self.images_loaded = false;
    }

    fn total_rows(&self) -> usize {
        (self.videos.len() + self.columns - 1) / self.columns
    }
}

impl Default for HomePage {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for HomePage {
    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(10),
                Constraint::Length(2),
            ])
            .split(area);

        // Header
        let title = format!(
            " Bilibili 推荐 | {} 个视频 | 第 {} 行 / {} 行 ",
            self.videos.len(),
            self.selected_row() + 1,
            self.total_rows()
        );
        let header = Paragraph::new(title)
            .block(Block::default().borders(Borders::ALL).title("首页"))
            .style(Style::default().fg(Color::Cyan))
            .alignment(Alignment::Center);
        frame.render_widget(header, chunks[0]);

        // Video grid
        if self.loading {
            let loading = Paragraph::new("加载中...")
                .style(Style::default().fg(Color::Yellow))
                .alignment(Alignment::Center);
            frame.render_widget(loading, chunks[1]);
        } else if let Some(ref error) = self.error_message {
            let error_widget = Paragraph::new(error.as_str())
                .style(Style::default().fg(Color::Red))
                .alignment(Alignment::Center);
            frame.render_widget(error_widget, chunks[1]);
        } else if self.videos.is_empty() {
            let empty = Paragraph::new("暂无推荐视频")
                .style(Style::default().fg(Color::Gray))
                .alignment(Alignment::Center);
            frame.render_widget(empty, chunks[1]);
        } else {
            self.render_grid(frame, chunks[1]);
        }

        // Help
        let help = Paragraph::new("←/h ↑/k ↓/j →/l 导航 | Enter 播放 | r 刷新 | q 退出")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        frame.render_widget(help, chunks[2]);
    }

    fn handle_input(&mut self, key: KeyCode) -> Option<AppAction> {
        match key {
            KeyCode::Char('q') => Some(AppAction::Quit),
            KeyCode::Char('j') | KeyCode::Down => {
                if !self.videos.is_empty() {
                    let new_idx = self.selected_index + self.columns;
                    if new_idx < self.videos.len() {
                        self.selected_index = new_idx;
                    }
                    self.update_scroll(3);
                }
                Some(AppAction::None)
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if !self.videos.is_empty() && self.selected_index >= self.columns {
                    self.selected_index -= self.columns;
                    self.update_scroll(3);
                }
                Some(AppAction::None)
            }
            KeyCode::Char('l') | KeyCode::Right => {
                if !self.videos.is_empty() && self.selected_index + 1 < self.videos.len() {
                    self.selected_index += 1;
                    self.update_scroll(3);
                }
                Some(AppAction::None)
            }
            KeyCode::Char('h') | KeyCode::Left => {
                if !self.videos.is_empty() && self.selected_index > 0 {
                    self.selected_index -= 1;
                    self.update_scroll(3);
                }
                Some(AppAction::None)
            }
            KeyCode::Enter => {
                if let Some(card) = self.videos.get(self.selected_index) {
                    if let Some(bvid) = &card.video.bvid {
                        return Some(AppAction::PlayVideo(bvid.clone()));
                    }
                }
                Some(AppAction::None)
            }
            KeyCode::Char('r') => {
                self.loading = true;
                self.videos.clear();
                Some(AppAction::SwitchToHome)
            }
            _ => Some(AppAction::None),
        }
    }
}

impl HomePage {
    fn render_grid(&mut self, frame: &mut Frame, area: Rect) {
        let visible_rows = self.visible_rows(area.height);
        let card_width = area.width / self.columns as u16;
        
        let row_constraints: Vec<Constraint> = (0..visible_rows)
            .map(|_| Constraint::Length(self.card_height))
            .collect();
        
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints(row_constraints)
            .split(area);

        // Collect all card areas first
        let mut card_areas: Vec<(usize, Rect)> = Vec::new();
        
        for (row_offset, row_area) in rows.iter().enumerate() {
            let actual_row = self.scroll_row + row_offset;
            let start_idx = actual_row * self.columns;
            
            if start_idx >= self.videos.len() {
                break;
            }

            let col_constraints: Vec<Constraint> = (0..self.columns)
                .map(|_| Constraint::Length(card_width))
                .collect();
            
            let cols = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(col_constraints)
                .split(*row_area);

            for (col_idx, col_area) in cols.iter().enumerate() {
                let video_idx = start_idx + col_idx;
                if video_idx >= self.videos.len() {
                    break;
                }
                card_areas.push((video_idx, *col_area));
            }
        }

        // Now render each card with mutable access
        for (video_idx, col_area) in card_areas {
            let is_selected = video_idx == self.selected_index;
            self.render_video_card(frame, col_area, video_idx, is_selected);
        }
    }

    fn render_video_card(&mut self, frame: &mut Frame, area: Rect, video_idx: usize, is_selected: bool) {
        let border_style = if is_selected {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(border_style)
            .title(if is_selected { "▶" } else { "" });
        
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let card_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(4),
                Constraint::Length(4),
            ])
            .split(inner);

        // Cover area - render with StatefulImage
        let cover_area = card_chunks[0];
        if let Some(ref mut cover) = self.videos[video_idx].cover {
            // Render actual image using StatefulImage
            let image_widget = StatefulImage::new();
            frame.render_stateful_widget(image_widget, cover_area, cover);
        } else {
            // Loading placeholder
            let placeholder = Paragraph::new("📺 加载中...")
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Center);
            frame.render_widget(placeholder, cover_area);
        }

        // Video info
        let info_area = card_chunks[1];
        let card = &self.videos[video_idx];
        
        let title = card.video.title.as_deref().unwrap_or("无标题");
        let author = card.video.author_name();
        let views = card.video.format_views();
        let duration = card.video.format_duration();

        let max_title_len = (info_area.width as usize).saturating_sub(2);
        let display_title: String = if title.chars().count() > max_title_len {
            title.chars().take(max_title_len.saturating_sub(3)).collect::<String>() + "..."
        } else {
            title.to_string()
        };

        let info_text = format!(
            "{}\n{}\n{} · {}",
            display_title, author, views, duration
        );

        let title_style = if is_selected {
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        let info = Paragraph::new(info_text)
            .style(title_style)
            .wrap(Wrap { trim: true });
        frame.render_widget(info, info_area);
    }
}
