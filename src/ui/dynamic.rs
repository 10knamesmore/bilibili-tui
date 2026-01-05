//! Dynamic feed page with video card grid display

use super::video_card::{VideoCard, VideoCardGrid};
use super::Component;
use crate::api::client::ApiClient;
use crate::api::dynamic::DynamicItem;
use crate::app::AppAction;
use ratatui::{crossterm::event::KeyCode, prelude::*, widgets::*};
use std::collections::HashMap;

/// Dynamic feed tab types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DynamicTab {
    /// All dynamics (视频+图文+文字)
    All,
    /// Video dynamics only
    Videos,
    /// Image dynamics (带图动态)
    Images,
    /// Text/Opus dynamics (图文动态)
    Text,
}

impl DynamicTab {
    pub fn label(&self) -> &str {
        match self {
            DynamicTab::All => "全部",
            DynamicTab::Videos => "视频",
            DynamicTab::Images => "图片",
            DynamicTab::Text => "图文",
        }
    }

    pub fn all_tabs() -> [DynamicTab; 4] {
        [
            DynamicTab::All,
            DynamicTab::Videos,
            DynamicTab::Images,
            DynamicTab::Text,
        ]
    }

    /// Get the API feed type parameter for this tab
    pub fn get_feed_type(&self) -> Option<&str> {
        match self {
            DynamicTab::All => None, // No type filter = all types
            DynamicTab::Videos => Some("video"),
            DynamicTab::Images => Some("draw"),
            DynamicTab::Text => Some("article"),
        }
    }
}

pub struct DynamicPage {
    pub grid: VideoCardGrid,
    pub loading: bool,
    pub error_message: Option<String>,
    pub offset: Option<String>,
    pub has_more: bool,
    pub loading_more: bool,
    pub current_tab: DynamicTab,
    pub tab_offsets: HashMap<DynamicTab, Option<String>>,
    pub up_list: Vec<crate::api::dynamic::UpListItem>,
    pub selected_up_index: usize, // 0 = "全部动态", 1+ = specific UP
    pub loading_up_list: bool,
}

impl DynamicPage {
    pub fn new() -> Self {
        Self {
            grid: VideoCardGrid::new(),
            loading: true,
            error_message: None,
            offset: None,
            has_more: false,
            loading_more: false,
            current_tab: DynamicTab::All,
            tab_offsets: HashMap::new(),
            up_list: Vec::new(),
            selected_up_index: 0,
            loading_up_list: false,
        }
    }

    pub fn set_up_list(&mut self, up_list: Vec<crate::api::dynamic::UpListItem>) {
        self.up_list = up_list;
        self.loading_up_list = false;
    }

    pub fn select_up(&mut self, index: usize) {
        if index <= self.up_list.len() {
            self.selected_up_index = index;
            self.grid.clear();
            self.loading = true;
        }
    }

    pub fn get_selected_up_mid(&self) -> Option<i64> {
        if self.selected_up_index == 0 {
            None
        } else {
            self.up_list.get(self.selected_up_index - 1).map(|u| u.mid)
        }
    }

    pub fn switch_tab(&mut self, tab: DynamicTab) {
        if self.current_tab != tab {
            self.current_tab = tab;
            self.offset = self.tab_offsets.get(&tab).cloned().flatten();
            self.grid.clear();
            self.loading = true;
            self.error_message = None;
        }
    }

    pub fn set_feed(&mut self, items: Vec<DynamicItem>, offset: Option<String>, has_more: bool) {
        self.grid.clear();

        // Process items based on current tab filter
        for item in items.into_iter() {
            let should_include = match self.current_tab {
                DynamicTab::All => item.is_video() || item.is_draw() || item.is_opus(),
                DynamicTab::Videos => item.is_video(),
                DynamicTab::Images => item.is_draw(),
                DynamicTab::Text => item.is_opus(),
            };

            if !should_include {
                continue;
            }

            // Handle video dynamics
            if item.is_video() {
                if let Some(bvid) = item.video_bvid() {
                    let card = VideoCard::new(
                        Some(bvid.to_string()),
                        None,
                        item.video_title().unwrap_or("无标题").to_string(),
                        item.author_name().to_string(),
                        format!("▶ {}", item.video_play()),
                        item.video_duration().to_string(),
                        item.video_cover().map(|s| s.to_string()),
                    );
                    self.grid.add_card(card);
                }
            }
            // Handle image dynamics (带图动态)
            else if item.is_draw() {
                let images = item.draw_images();
                let image_url = images.first().map(|s| s.to_string());
                let desc = item.desc_text().unwrap_or("图片动态");
                let image_count = if images.len() > 1 {
                    format!(" [{}P]", images.len())
                } else {
                    String::new()
                };

                let card = VideoCard::new(
                    None, // No bvid for images
                    None,
                    format!("{}{}", desc, image_count),
                    item.author_name().to_string(),
                    "📷 图片动态".to_string(),
                    "".to_string(),
                    image_url,
                );
                self.grid.add_card(card);
            }
            // Handle text/opus dynamics (图文动态)
            else if item.is_opus() {
                let text = item.opus_text().unwrap_or("图文动态");
                let images = item.opus_images();
                let image_url = images.first().map(|s| s.to_string());
                let image_count = if !images.is_empty() {
                    format!(" [{}P]", images.len())
                } else {
                    String::new()
                };

                let card = VideoCard::new(
                    None,
                    None,
                    format!("{}{}", text, image_count),
                    item.author_name().to_string(),
                    "📝 图文".to_string(),
                    "".to_string(),
                    image_url,
                );
                self.grid.add_card(card);
            }
        }

        // Save offset for current tab
        self.tab_offsets.insert(self.current_tab, offset.clone());
        self.offset = offset;
        self.has_more = has_more;
        self.loading = false;
    }

    pub fn append_feed(&mut self, items: Vec<DynamicItem>, offset: Option<String>, has_more: bool) {
        // Process items based on current tab filter
        for item in items.into_iter() {
            let should_include = match self.current_tab {
                DynamicTab::All => item.is_video() || item.is_draw() || item.is_opus(),
                DynamicTab::Videos => item.is_video(),
                DynamicTab::Images => item.is_draw(),
                DynamicTab::Text => item.is_opus(),
            };

            if !should_include {
                continue;
            }

            // Handle video dynamics
            if item.is_video() {
                if let Some(bvid) = item.video_bvid() {
                    let card = VideoCard::new(
                        Some(bvid.to_string()),
                        None,
                        item.video_title().unwrap_or("无标题").to_string(),
                        item.author_name().to_string(),
                        format!("▶ {}", item.video_play()),
                        item.video_duration().to_string(),
                        item.video_cover().map(|s| s.to_string()),
                    );
                    self.grid.add_card(card);
                }
            }
            // Handle image dynamics
            else if item.is_draw() {
                let images = item.draw_images();
                let image_url = images.first().map(|s| s.to_string());
                let desc = item.desc_text().unwrap_or("图片动态");
                let image_count = if images.len() > 1 {
                    format!(" [{}P]", images.len())
                } else {
                    String::new()
                };

                let card = VideoCard::new(
                    None,
                    None,
                    format!("{}{}", desc, image_count),
                    item.author_name().to_string(),
                    "📷 图片动态".to_string(),
                    "".to_string(),
                    image_url,
                );
                self.grid.add_card(card);
            }
            // Handle text/opus dynamics
            else if item.is_opus() {
                let text = item.opus_text().unwrap_or("图文动态");
                let images = item.opus_images();
                let image_url = images.first().map(|s| s.to_string());
                let image_count = if !images.is_empty() {
                    format!(" [{}P]", images.len())
                } else {
                    String::new()
                };

                let card = VideoCard::new(
                    None,
                    None,
                    format!("{}{}", text, image_count),
                    item.author_name().to_string(),
                    "📝 图文".to_string(),
                    "".to_string(),
                    image_url,
                );
                self.grid.add_card(card);
            }
        }

        // Save offset for current tab
        self.tab_offsets.insert(self.current_tab, offset.clone());
        self.offset = offset;
        self.has_more = has_more;
        self.loading_more = false;
    }

    pub fn set_error(&mut self, msg: String) {
        self.error_message = Some(msg);
        self.loading = false;
        self.loading_more = false;
    }

    pub async fn load_more(&mut self, api_client: &ApiClient) {
        if self.loading_more || !self.has_more {
            return;
        }

        self.loading_more = true;

        let feed_type = self.current_tab.get_feed_type();
        let host_mid = self.get_selected_up_mid();
        match api_client
            .get_dynamic_feed(self.offset.as_deref(), feed_type, host_mid)
            .await
        {
            Ok(data) => {
                let items = data.items.unwrap_or_default();
                let offset = data.offset;
                let has_more = data.has_more.unwrap_or(false);
                self.append_feed(items, offset, has_more);
            }
            Err(_) => {
                self.loading_more = false;
            }
        }
    }

    pub fn poll_cover_results(&mut self) {
        self.grid.poll_cover_results();
    }

    pub fn start_cover_downloads(&mut self) {
        self.grid.start_cover_downloads();
    }
}

impl Default for DynamicPage {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for DynamicPage {
    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // UP master selection bar
                Constraint::Length(5), // Header with tabs
                Constraint::Min(10),   // Grid
                Constraint::Length(2), // Help
            ])
            .split(area);

        // UP master selection bar
        let mut up_spans = vec![if self.selected_up_index == 0 {
            Span::styled(
                " [全部] ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
                    .add_modifier(Modifier::UNDERLINED),
            )
        } else {
            Span::styled(" [全部] ", Style::default().fg(Color::Rgb(120, 120, 120)))
        }];

        for (i, user) in self.up_list.iter().enumerate().take(10) {
            let is_selected = self.selected_up_index == i + 1;
            let name = &user.uname;
            // Add update indicator (●) for UPs with recent updates
            let text = if user.has_update {
                format!(" ● {} ", name)
            } else {
                format!(" {} ", name)
            };

            if is_selected {
                up_spans.push(Span::styled(
                    text,
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                        .add_modifier(Modifier::UNDERLINED),
                ));
            } else {
                let color = if user.has_update {
                    Color::LightBlue // Light blue for unselected with update
                } else {
                    Color::Rgb(150, 150, 150) // Gray for no update
                };
                up_spans.push(Span::styled(text, Style::default().fg(color)));
            }
        }

        if self.up_list.len() > 10 {
            up_spans.push(Span::styled(
                " ...更多 ",
                Style::default().fg(Color::Rgb(100, 100, 100)),
            ));
        }

        let up_bar = Paragraph::new(Line::from(up_spans))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title("关注的UP主")
                    .border_style(Style::default().fg(Color::Rgb(60, 60, 60))),
            )
            .alignment(Alignment::Left);
        frame.render_widget(up_bar, chunks[0]);

        // Header with tab bar
        let header_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // Title line
                Constraint::Length(3), // Tab bar
            ])
            .split(chunks[1]);

        // Title
        let title = Paragraph::new(Line::from(vec![
            Span::styled(" 📺 ", Style::default()),
            Span::styled(
                "关注动态",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(" ({} 条)", self.grid.cards.len()),
                Style::default().fg(Color::Rgb(100, 100, 100)),
            ),
            if self.loading_more {
                Span::styled(" 加载中...", Style::default().fg(Color::Yellow))
            } else {
                Span::raw("")
            },
        ]))
        .block(
            Block::default()
                .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Rgb(60, 60, 60))),
        )
        .alignment(Alignment::Center);
        frame.render_widget(title, header_chunks[0]);

        // Tab bar
        let mut tab_spans = Vec::new();
        for (i, tab) in DynamicTab::all_tabs().iter().enumerate() {
            if i > 0 {
                tab_spans.push(Span::raw("  "));
            }

            let is_active = *tab == self.current_tab;
            let tab_text = format!("[{}] {}", i + 1, tab.label());

            if is_active {
                tab_spans.push(Span::styled(
                    tab_text,
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                        .add_modifier(Modifier::UNDERLINED),
                ));
            } else {
                tab_spans.push(Span::styled(
                    tab_text,
                    Style::default().fg(Color::Rgb(120, 120, 120)),
                ));
            }
        }

        let tabs = Paragraph::new(Line::from(tab_spans))
            .block(
                Block::default()
                    .borders(Borders::BOTTOM | Borders::LEFT | Borders::RIGHT)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::Rgb(60, 60, 60))),
            )
            .alignment(Alignment::Center);
        frame.render_widget(tabs, header_chunks[1]);

        // Content
        if self.loading {
            let loading = Paragraph::new("⏳ 加载动态中...")
                .style(Style::default().fg(Color::Yellow))
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .border_style(Style::default().fg(Color::Rgb(60, 60, 60))),
                );
            frame.render_widget(loading, chunks[2]);
        } else if let Some(ref error) = self.error_message {
            let error_widget = Paragraph::new(format!("❌ {}", error))
                .style(Style::default().fg(Color::Red))
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .border_style(Style::default().fg(Color::Rgb(60, 60, 60))),
                );
            frame.render_widget(error_widget, chunks[2]);
        } else if self.grid.cards.is_empty() {
            let empty = Paragraph::new("暂无动态，请先登录并关注UP主")
                .style(Style::default().fg(Color::Rgb(100, 100, 100)))
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .border_style(Style::default().fg(Color::Rgb(60, 60, 60))),
                );
            frame.render_widget(empty, chunks[2]);
        } else {
            self.grid.render(frame, chunks[2]);
        }

        // Help
        let help = Paragraph::new("↑↓←→/hjkl:卡片导航 | Tab/Shift+Tab:切UP主 | []:切标签 | 1-4:直达 | Enter:详情 | r:刷新 | n:切页面")
            .style(Style::default().fg(Color::Rgb(80, 80, 80)))
            .alignment(Alignment::Center);
        frame.render_widget(help, chunks[3]);
    }

    fn handle_input_with_modifiers(
        &mut self,
        key: KeyCode,
        modifiers: crossterm::event::KeyModifiers,
    ) -> Option<AppAction> {
        use crossterm::event::KeyModifiers;

        match (key, modifiers) {
            // Card navigation - Arrow keys
            (KeyCode::Down, KeyModifiers::NONE) => {
                self.grid.move_down();
                if self.grid.is_near_bottom(3) && !self.loading_more && self.has_more {
                    return Some(AppAction::LoadMoreDynamic);
                }
                Some(AppAction::None)
            }
            (KeyCode::Up, KeyModifiers::NONE) => {
                self.grid.move_up();
                Some(AppAction::None)
            }
            (KeyCode::Left, KeyModifiers::NONE) => {
                self.grid.move_left();
                Some(AppAction::None)
            }
            (KeyCode::Right, KeyModifiers::NONE) => {
                self.grid.move_right();
                Some(AppAction::None)
            }

            // Card navigation - vim keys (hjkl)
            (KeyCode::Char('j'), KeyModifiers::NONE) => {
                self.grid.move_down();
                if self.grid.is_near_bottom(3) && !self.loading_more && self.has_more {
                    return Some(AppAction::LoadMoreDynamic);
                }
                Some(AppAction::None)
            }
            (KeyCode::Char('k'), KeyModifiers::NONE) => {
                self.grid.move_up();
                Some(AppAction::None)
            }
            (KeyCode::Char('h'), KeyModifiers::NONE) => {
                self.grid.move_left();
                Some(AppAction::None)
            }
            (KeyCode::Char('l'), KeyModifiers::NONE) => {
                self.grid.move_right();
                Some(AppAction::None)
            }

            // UP master navigation - Shift+Tab (previous), Tab (next)
            (KeyCode::BackTab, _) => {
                if self.selected_up_index > 0 {
                    Some(AppAction::SelectUpMaster(self.selected_up_index - 1))
                } else {
                    Some(AppAction::None)
                }
            }
            (KeyCode::Tab, KeyModifiers::NONE) => {
                if self.selected_up_index < self.up_list.len() {
                    Some(AppAction::SelectUpMaster(self.selected_up_index + 1))
                } else {
                    Some(AppAction::None)
                }
            }

            // Tab switching - [ and ] keys
            (KeyCode::Char('['), KeyModifiers::NONE) => {
                let new_tab = match self.current_tab {
                    DynamicTab::All => DynamicTab::Text,
                    DynamicTab::Videos => DynamicTab::All,
                    DynamicTab::Images => DynamicTab::Videos,
                    DynamicTab::Text => DynamicTab::Images,
                };
                Some(AppAction::SwitchDynamicTab(new_tab))
            }
            (KeyCode::Char(']'), KeyModifiers::NONE) => {
                let new_tab = match self.current_tab {
                    DynamicTab::All => DynamicTab::Videos,
                    DynamicTab::Videos => DynamicTab::Images,
                    DynamicTab::Images => DynamicTab::Text,
                    DynamicTab::Text => DynamicTab::All,
                };
                Some(AppAction::SwitchDynamicTab(new_tab))
            }

            // Tab switching - number keys (1-4) for direct access
            (KeyCode::Char('1'), KeyModifiers::NONE) => {
                Some(AppAction::SwitchDynamicTab(DynamicTab::All))
            }
            (KeyCode::Char('2'), KeyModifiers::NONE) => {
                Some(AppAction::SwitchDynamicTab(DynamicTab::Videos))
            }
            (KeyCode::Char('3'), KeyModifiers::NONE) => {
                Some(AppAction::SwitchDynamicTab(DynamicTab::Images))
            }
            (KeyCode::Char('4'), KeyModifiers::NONE) => {
                Some(AppAction::SwitchDynamicTab(DynamicTab::Text))
            }

            // Open selected card
            (KeyCode::Enter, _) => {
                if let Some(card) = self.grid.selected_card() {
                    if let Some(ref bvid) = card.bvid {
                        return Some(AppAction::OpenVideoDetail(bvid.clone(), 0));
                    }
                }
                Some(AppAction::None)
            }

            // Refresh
            (KeyCode::Char('r'), KeyModifiers::NONE) => {
                self.loading = true;
                self.grid.clear();
                Some(AppAction::RefreshDynamic)
            }

            // Navigate to next sidebar item
            (KeyCode::Char('n'), KeyModifiers::NONE) => Some(AppAction::NavNext),

            // Quit
            (KeyCode::Char('q'), KeyModifiers::NONE) => Some(AppAction::Quit),

            _ => Some(AppAction::None),
        }
    }
}
