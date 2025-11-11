mod app;
mod audio;
mod entities;
mod input;
mod renderer;

use app::App;
use crossterm::{
    event::{KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io::Write;
use std::{fs::OpenOptions, io::stdout};

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let supports_keyboard_enhancement = matches!(
        crossterm::terminal::supports_keyboard_enhancement(),
        Ok(true)
    );

    // Debug: Log keyboard enhancement support
    let mut debug_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("debug.log")?;
    writeln!(
        debug_file,
        "Keyboard enhancement supported: {}",
        supports_keyboard_enhancement
    )?;

    // Setup terminal manually for full control
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;

    // Enable keyboard enhancement AFTER entering alternate screen
    if supports_keyboard_enhancement {
        execute!(
            stdout,
            PushKeyboardEnhancementFlags(
                KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
                    | KeyboardEnhancementFlags::REPORT_EVENT_TYPES
            )
        )?;
        writeln!(debug_file, "Keyboard enhancement flags pushed")?;
    }

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = App::new().run(&mut terminal);

    // Cleanup
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    if supports_keyboard_enhancement {
        execute!(terminal.backend_mut(), PopKeyboardEnhancementFlags)?;
    }

    terminal.show_cursor()?;

    result
}
