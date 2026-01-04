mod action;

pub use action::AppAction;

use crate::api::client::ApiClient;
use crate::storage::Credentials;
use crate::ui::{Component, HomePage, LoginPage, Page};
use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    DefaultTerminal, Frame,
};
use std::io;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Main application state
pub struct App {
    pub current_page: Page,
    pub should_quit: bool,
    pub api_client: Arc<Mutex<ApiClient>>,
    pub credentials: Option<Credentials>,
}

impl App {
    pub fn new() -> Self {
        let credentials = crate::storage::load_credentials().ok();
        let api_client = if let Some(ref creds) = credentials {
            ApiClient::with_cookies(creds)
        } else {
            ApiClient::new()
        };

        // Start on login page if no credentials, otherwise go to home
        let current_page = if credentials.is_some() {
            Page::Home(HomePage::new())
        } else {
            Page::Login(LoginPage::new())
        };

        Self {
            current_page,
            should_quit: false,
            api_client: Arc::new(Mutex::new(api_client)),
            credentials,
        }
    }

    /// Main run loop
    pub async fn run(mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        // Initialize the first page
        self.init_current_page().await;

        while !self.should_quit {
            terminal.draw(|frame| self.draw(frame))?;

            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        self.handle_input(key.code).await;
                    }
                }
            }

            // Handle background tasks (like QR code polling)
            self.tick().await;
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();
        match &mut self.current_page {
            Page::Login(page) => page.draw(frame, area),
            Page::Home(page) => page.draw(frame, area),
        }
    }

    async fn handle_input(&mut self, key: KeyCode) {
        let action = match &mut self.current_page {
            Page::Login(page) => page.handle_input(key),
            Page::Home(page) => page.handle_input(key),
        };

        if let Some(action) = action {
            self.handle_action(action).await;
        }
    }

    async fn handle_action(&mut self, action: AppAction) {
        match action {
            AppAction::Quit => self.should_quit = true,
            AppAction::SwitchToHome => {
                self.current_page = Page::Home(HomePage::new());
                self.init_current_page().await;
            }
            AppAction::SwitchToLogin => {
                self.current_page = Page::Login(LoginPage::new());
                self.init_current_page().await;
            }
            AppAction::LoginSuccess(creds) => {
                // Save credentials
                if let Err(e) = crate::storage::save_credentials(&creds) {
                    eprintln!("Failed to save credentials: {}", e);
                }
                self.credentials = Some(creds.clone());
                // Update API client with new cookies
                {
                    let client = self.api_client.lock().await;
                    client.set_credentials(&creds);
                }
                // Switch to home
                self.current_page = Page::Home(HomePage::new());
                self.init_current_page().await;
            }
            AppAction::PlayVideo(bvid) => {
                // Launch mpv player
                if let Err(e) = crate::player::play_video(&bvid, self.credentials.as_ref()).await {
                    eprintln!("Failed to play video: {}", e);
                }
            }
            AppAction::None => {}
        }
    }

    async fn init_current_page(&mut self) {
        match &mut self.current_page {
            Page::Login(page) => {
                let client = self.api_client.lock().await;
                page.load_qrcode(&client).await;
            }
            Page::Home(page) => {
                let client = self.api_client.lock().await;
                page.load_recommendations(&client).await;
            }
        }
    }

    async fn tick(&mut self) {
        match &mut self.current_page {
            Page::Login(page) => {
                let client = self.api_client.lock().await;
                if let Some(action) = page.tick(&client).await {
                    drop(client);
                    self.handle_action(action).await;
                }
            }
            Page::Home(page) => {
                // Load visible cover images in background
                page.load_visible_covers().await;
            }
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
