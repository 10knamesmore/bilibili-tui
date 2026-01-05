//! Left sidebar navigation component

use ratatui::{prelude::*, widgets::*};

/// Navigation menu items
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavItem {
    Home,
    Search,
    Dynamic,
}

impl NavItem {
    pub fn label(&self) -> &'static str {
        match self {
            NavItem::Home => "🏠 首页",
            NavItem::Search => "🔍 搜索",
            NavItem::Dynamic => "📺 动态",
        }
    }

    pub fn all() -> &'static [NavItem] {
        &[NavItem::Home, NavItem::Search, NavItem::Dynamic]
    }
}

pub struct Sidebar {
    pub selected: NavItem,
}

impl Sidebar {
    pub fn new() -> Self {
        Self {
            selected: NavItem::Home,
        }
    }

    pub fn draw(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::RIGHT)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Rgb(50, 50, 50)))
            .title(Span::styled(
                " 导航 ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let items: Vec<ListItem> = NavItem::all()
            .iter()
            .map(|item| {
                let is_selected = *item == self.selected;
                let style = if is_selected {
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                        .bg(Color::Rgb(40, 40, 40))
                } else {
                    Style::default().fg(Color::Rgb(150, 150, 150))
                };

                let prefix = if is_selected { "▶ " } else { "  " };
                ListItem::new(format!("{}{}", prefix, item.label())).style(style)
            })
            .collect();

        let list = List::new(items).highlight_style(Style::default().add_modifier(Modifier::BOLD));

        frame.render_widget(list, inner);
    }

    pub fn next(&mut self) {
        let items = NavItem::all();
        let current_idx = items.iter().position(|i| *i == self.selected).unwrap_or(0);
        let next_idx = (current_idx + 1) % items.len();
        self.selected = items[next_idx];
    }

    pub fn prev(&mut self) {
        let items = NavItem::all();
        let current_idx = items.iter().position(|i| *i == self.selected).unwrap_or(0);
        let prev_idx = if current_idx == 0 {
            items.len() - 1
        } else {
            current_idx - 1
        };
        self.selected = items[prev_idx];
    }

    pub fn select(&mut self, item: NavItem) {
        self.selected = item;
    }
}

impl Default for Sidebar {
    fn default() -> Self {
        Self::new()
    }
}
