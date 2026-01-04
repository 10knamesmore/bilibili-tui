use crate::storage::Credentials;

/// Actions that can be triggered from UI components
#[derive(Debug, Clone)]
pub enum AppAction {
    /// Quit the application
    Quit,
    /// Switch to home page
    SwitchToHome,
    /// Switch to login page
    SwitchToLogin,
    /// Login was successful with credentials
    LoginSuccess(Credentials),
    /// Play a video by bvid
    PlayVideo(String),
    /// Navigate to next sidebar item
    NavNext,
    /// Navigate to previous sidebar item
    NavPrev,
    /// Search for videos
    Search(String),
    /// Refresh dynamic feed
    RefreshDynamic,
    /// No action
    None,
}
