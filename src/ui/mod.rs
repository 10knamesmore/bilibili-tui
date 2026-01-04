mod home;
mod login;

pub use home::HomePage;
pub use login::LoginPage;

use crate::app::AppAction;
use ratatui::{crossterm::event::KeyCode, Frame, prelude::Rect};

/// UI Component trait
pub trait Component {
    fn draw(&mut self, frame: &mut Frame, area: Rect);
    fn handle_input(&mut self, key: KeyCode) -> Option<AppAction>;
}

/// Application pages
pub enum Page {
    Login(LoginPage),
    Home(HomePage),
}
