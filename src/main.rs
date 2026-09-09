use std::{io, process::Command, time::Duration};

use agentman::{app::{App, UiAction}, discover_all, launch_command, rename_session, ui};
use crossterm::{event::{self, Event, KeyCode, MouseEventKind}, execute, terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}};
use ratatui::{Terminal, backend::CrosstermBackend};

fn main() -> anyhow::Result<()> {
    if std::env::args().any(|argument| argument == "--version" || argument == "-V") {
        println!("agentman {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    if std::env::args().any(|argument| argument == "--help" || argument == "-h") {
        println!("agentman\n\nInteractive local coding-agent session manager.\n\nShortcuts: Enter resume · f fork · y/Y YOLO · r rename · d move to Trash · / search · q quit");
        return Ok(());
    }
    let home = std::env::var_os("HOME").ok_or_else(|| anyhow::anyhow!("HOME is not set"))?;
    let mut app = App::new(discover_all(std::path::Path::new(&home))?);
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, event::EnableMouseCapture)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;
    let result = run(&mut terminal, &mut app);
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), event::DisableMouseCapture, LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    result
}

fn run(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> anyhow::Result<()> {
    loop {
        terminal.draw(|frame| ui::render(frame, app))?;
        if !event::poll(Duration::from_millis(120))? { continue; }
        match event::read()? {
            Event::Key(key) => match key.code {
                KeyCode::Char('q') if !app.searching() && !app.confirming_delete() && !app.renaming() => break,
                KeyCode::Up => app.move_selection(-1), KeyCode::Down => app.move_selection(1),
                KeyCode::Enter => app.key('\n'), KeyCode::Backspace => app.key('\u{8}'), KeyCode::Esc => app.key('\u{1b}'),
                KeyCode::Char(character) => app.key(character), _ => {}
            },
            Event::Mouse(mouse) => match mouse.kind { MouseEventKind::ScrollDown => app.move_selection(1), MouseEventKind::ScrollUp => app.move_selection(-1), _ => {} },
            _ => {}
        }
        if let Some(action) = app.take_action() { execute_action(terminal, app, action)?; }
    }
    Ok(())
}

fn execute_action(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App, action: UiAction) -> anyhow::Result<()> {
    match action {
        UiAction::Trash(index) => { trash::delete(&app.sessions[index].path)?; app.sessions.remove(index); app.selected = 0; }
        UiAction::Rename(index, title) if !title.is_empty() => { rename_session(&app.sessions[index], &title)?; app.sessions[index].title = title; }
        UiAction::Rename(_, _) => {}
        UiAction::Launch(index, action) => {
            let Some(command) = launch_command(&app.sessions[index], action) else { return Ok(()) };
            disable_raw_mode()?;
            execute!(terminal.backend_mut(), event::DisableMouseCapture, LeaveAlternateScreen)?;
            let _ = Command::new(command.program).args(command.args).status();
            execute!(terminal.backend_mut(), EnterAlternateScreen, event::EnableMouseCapture)?;
            enable_raw_mode()?;
        }
    }
    Ok(())
}
