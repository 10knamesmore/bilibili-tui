use bilibili_tui::app::App;
use std::io;

#[tokio::main]
async fn main() -> io::Result<()> {
    // Initialize terminal
    let mut terminal = ratatui::init();
    terminal.clear()?;

    // Run the application
    let app = App::new();
    let result = app.run(&mut terminal).await;

    // Restore terminal
    ratatui::restore();

    result
}
