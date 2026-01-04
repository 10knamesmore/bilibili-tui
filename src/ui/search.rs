//! Search page with input and results

use super::Component;
use crate::api::search::SearchVideoItem;
use crate::app::AppAction;
use ratatui::{
    crossterm::event::KeyCode,
    prelude::*,
    widgets::*,
};

pub struct SearchPage {
    pub query: String,
    pub results: Vec<SearchVideoItem>,
    pub selected_index: usize,
    pub loading: bool,
    pub error_message: Option<String>,
    pub input_mode: bool,
    pub page: i32,
    pub total_results: i32,
}

impl SearchPage {
    pub fn new() -> Self {
        Self {
            query: String::new(),
            results: Vec::new(),
            selected_index: 0,
            loading: false,
            error_message: None,
            input_mode: true,
            page: 1,
            total_results: 0,
        }
    }

    pub fn set_results(&mut self, results: Vec<SearchVideoItem>, total: i32) {
        self.results = results;
        self.total_results = total;
        self.selected_index = 0;
        self.loading = false;
        self.input_mode = false;
    }

    pub fn set_error(&mut self, msg: String) {
        self.error_message = Some(msg);
        self.loading = false;
    }
}

impl Default for SearchPage {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for SearchPage {
    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Search input
                Constraint::Min(5),     // Results
                Constraint::Length(2),  // Help
            ])
            .split(area);

        // Search input
        let input_style = if self.input_mode {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::White)
        };
        
        let input_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(if self.input_mode {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::Rgb(60, 60, 60))
            })
            .title(Span::styled(" 🔍 搜索视频 ", Style::default().fg(Color::Cyan)));

        let cursor_char = if self.input_mode { "▌" } else { "" };
        let input = Paragraph::new(format!("{}{}", self.query, cursor_char))
            .style(input_style)
            .block(input_block);
        frame.render_widget(input, chunks[0]);

        // Results
        let results_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Rgb(60, 60, 60)))
            .title(Span::styled(
                format!(" 结果 ({}) ", self.total_results),
                Style::default().fg(Color::Rgb(150, 150, 150))
            ));

        if self.loading {
            let loading = Paragraph::new("⏳ 搜索中...")
                .style(Style::default().fg(Color::Yellow))
                .alignment(Alignment::Center)
                .block(results_block);
            frame.render_widget(loading, chunks[1]);
        } else if let Some(ref error) = self.error_message {
            let error_widget = Paragraph::new(format!("❌ {}", error))
                .style(Style::default().fg(Color::Red))
                .alignment(Alignment::Center)
                .block(results_block);
            frame.render_widget(error_widget, chunks[1]);
        } else if self.results.is_empty() {
            let empty = Paragraph::new(if self.query.is_empty() {
                "输入关键词开始搜索"
            } else {
                "没有找到相关视频"
            })
                .style(Style::default().fg(Color::Rgb(100, 100, 100)))
                .alignment(Alignment::Center)
                .block(results_block);
            frame.render_widget(empty, chunks[1]);
        } else {
            let inner = results_block.inner(chunks[1]);
            frame.render_widget(results_block, chunks[1]);

            let items: Vec<ListItem> = self.results
                .iter()
                .enumerate()
                .map(|(i, item)| {
                    let is_selected = i == self.selected_index;
                    let style = if is_selected {
                        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::White)
                    };
                    
                    let prefix = if is_selected { "▶ " } else { "  " };
                    let title = item.display_title();
                    let author = item.author_name().to_string();
                    let play = item.format_play();
                    
                    ListItem::new(Line::from(vec![
                        Span::styled(prefix, style),
                        Span::styled(title, style),
                        Span::styled(format!("  {} · {}", author, play), Style::default().fg(Color::Rgb(100, 100, 100))),
                    ]))
                })
                .collect();

            let list = List::new(items);
            frame.render_widget(list, inner);
        }

        // Help
        let help_text = if self.input_mode {
            "[Enter] 搜索  [Esc] 取消  [Tab] 导航"
        } else {
            "[j/k] 上下  [Enter] 播放  [/] 输入  [Tab] 导航"
        };
        let help = Paragraph::new(help_text)
            .style(Style::default().fg(Color::Rgb(80, 80, 80)))
            .alignment(Alignment::Center);
        frame.render_widget(help, chunks[2]);
    }

    fn handle_input(&mut self, key: KeyCode) -> Option<AppAction> {
        if self.input_mode {
            match key {
                KeyCode::Char(c) => {
                    self.query.push(c);
                    Some(AppAction::None)
                }
                KeyCode::Backspace => {
                    self.query.pop();
                    Some(AppAction::None)
                }
                KeyCode::Enter => {
                    if !self.query.is_empty() {
                        self.loading = true;
                        self.page = 1;
                        Some(AppAction::Search(self.query.clone()))
                    } else {
                        Some(AppAction::None)
                    }
                }
                KeyCode::Esc => {
                    self.input_mode = false;
                    Some(AppAction::None)
                }
                KeyCode::Tab => Some(AppAction::NavNext),
                _ => Some(AppAction::None),
            }
        } else {
            match key {
                KeyCode::Char('j') | KeyCode::Down => {
                    if !self.results.is_empty() && self.selected_index + 1 < self.results.len() {
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
                    if let Some(item) = self.results.get(self.selected_index) {
                        if let Some(ref bvid) = item.bvid {
                            return Some(AppAction::PlayVideo(bvid.clone()));
                        }
                    }
                    Some(AppAction::None)
                }
                KeyCode::Char('/') | KeyCode::Char('i') => {
                    self.input_mode = true;
                    Some(AppAction::None)
                }
                KeyCode::Tab => Some(AppAction::NavNext),
                KeyCode::Char('q') => Some(AppAction::Quit),
                _ => Some(AppAction::None),
            }
        }
    }
}
