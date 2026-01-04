//! Dynamic feed page showing following updates

use super::Component;
use crate::api::dynamic::DynamicItem;
use crate::app::AppAction;
use ratatui::{
    crossterm::event::KeyCode,
    prelude::*,
    widgets::*,
};

pub struct DynamicPage {
    pub items: Vec<DynamicItem>,
    pub selected_index: usize,
    pub loading: bool,
    pub error_message: Option<String>,
    pub offset: Option<String>,
    pub has_more: bool,
}

impl DynamicPage {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            selected_index: 0,
            loading: true,
            error_message: None,
            offset: None,
            has_more: false,
        }
    }

    pub fn set_feed(&mut self, items: Vec<DynamicItem>, offset: Option<String>, has_more: bool) {
        // Filter only video dynamics
        self.items = items.into_iter().filter(|i| i.is_video()).collect();
        self.offset = offset;
        self.has_more = has_more;
        self.selected_index = 0;
        self.loading = false;
    }

    pub fn set_error(&mut self, msg: String) {
        self.error_message = Some(msg);
        self.loading = false;
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
                Constraint::Length(3),  // Header
                Constraint::Min(5),     // Feed
                Constraint::Length(2),  // Help
            ])
            .split(area);

        // Header
        let header = Paragraph::new(Line::from(vec![
            Span::styled(" 📺 ", Style::default()),
            Span::styled("关注动态", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(format!(" ({} 条)", self.items.len()), Style::default().fg(Color::Rgb(100, 100, 100))),
        ]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Rgb(60, 60, 60)))
        )
        .alignment(Alignment::Center);
        frame.render_widget(header, chunks[0]);

        // Feed list
        let feed_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Rgb(60, 60, 60)));

        if self.loading {
            let loading = Paragraph::new("⏳ 加载动态中...")
                .style(Style::default().fg(Color::Yellow))
                .alignment(Alignment::Center)
                .block(feed_block);
            frame.render_widget(loading, chunks[1]);
        } else if let Some(ref error) = self.error_message {
            let error_widget = Paragraph::new(format!("❌ {}", error))
                .style(Style::default().fg(Color::Red))
                .alignment(Alignment::Center)
                .block(feed_block);
            frame.render_widget(error_widget, chunks[1]);
        } else if self.items.is_empty() {
            let empty = Paragraph::new("暂无动态，请先登录并关注UP主")
                .style(Style::default().fg(Color::Rgb(100, 100, 100)))
                .alignment(Alignment::Center)
                .block(feed_block);
            frame.render_widget(empty, chunks[1]);
        } else {
            let inner = feed_block.inner(chunks[1]);
            frame.render_widget(feed_block, chunks[1]);

            // Calculate visible items
            let visible_height = inner.height as usize;
            let item_height = 3; // Each item takes 3 lines
            let visible_count = (visible_height / item_height).max(1);
            let scroll_offset = (self.selected_index / visible_count) * visible_count;

            let items: Vec<ListItem> = self.items
                .iter()
                .enumerate()
                .skip(scroll_offset)
                .take(visible_count)
                .map(|(i, item)| {
                    let is_selected = i == self.selected_index;
                    let style = if is_selected {
                        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::White)
                    };
                    
                    let prefix = if is_selected { "▶ " } else { "  " };
                    let author = item.author_name();
                    let time = item.pub_time();
                    let title = item.video_title().unwrap_or("无标题");
                    let play = item.video_play();
                    let danmaku = item.video_danmaku();
                    
                    let lines = vec![
                        Line::from(vec![
                            Span::styled(prefix, style),
                            Span::styled(author, Style::default().fg(Color::Rgb(251, 114, 153)).add_modifier(Modifier::BOLD)),
                            Span::styled(format!("  {}", time), Style::default().fg(Color::Rgb(80, 80, 80))),
                        ]),
                        Line::from(vec![
                            Span::raw("   "),
                            Span::styled(title, style),
                        ]),
                        Line::from(vec![
                            Span::raw("   "),
                            Span::styled(format!("▶ {} · 💬 {}", play, danmaku), Style::default().fg(Color::Rgb(100, 100, 100))),
                        ]),
                    ];
                    
                    ListItem::new(lines)
                })
                .collect();

            let list = List::new(items);
            frame.render_widget(list, inner);
        }

        // Help
        let help = Paragraph::new("[j/k] 上下  [Enter] 播放  [r] 刷新  [Tab] 导航")
            .style(Style::default().fg(Color::Rgb(80, 80, 80)))
            .alignment(Alignment::Center);
        frame.render_widget(help, chunks[2]);
    }

    fn handle_input(&mut self, key: KeyCode) -> Option<AppAction> {
        match key {
            KeyCode::Char('j') | KeyCode::Down => {
                if !self.items.is_empty() && self.selected_index + 1 < self.items.len() {
                    self.selected_index += 1;
                }
                Some(AppAction::None)
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
                Some(AppAction::None)
            }
            KeyCode::Enter => {
                if let Some(item) = self.items.get(self.selected_index) {
                    if let Some(bvid) = item.video_bvid() {
                        return Some(AppAction::PlayVideo(bvid.to_string()));
                    }
                }
                Some(AppAction::None)
            }
            KeyCode::Char('r') => {
                self.loading = true;
                self.items.clear();
                Some(AppAction::RefreshDynamic)
            }
            KeyCode::Tab => Some(AppAction::NavNext),
            KeyCode::Char('q') => Some(AppAction::Quit),
            _ => Some(AppAction::None),
        }
    }
}
