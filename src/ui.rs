use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table, TableState, Wrap},
};

use crate::app::App;

const CYAN: Color = Color::LightCyan;
const BLUE: Color = Color::Cyan;

#[allow(clippy::too_many_lines)]
pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(4),
            Constraint::Length(2),
        ])
        .split(area);
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(
                " ◉ AGENTMAN ",
                Style::default()
                    .fg(Color::Black)
                    .bg(CYAN)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" local coding-agent session manager"),
        ]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(BLUE)),
        ),
        layout[0],
    );
    let indices = app.matching();
    let rows = indices.iter().map(|index| {
        let session = &app.sessions[*index];
        let project = session
            .project
            .as_ref()
            .map_or_else(|| "—".to_owned(), |path| path.display().to_string());
        Row::new(vec![
            Cell::from(session.agent.label()),
            Cell::from(session.title.clone()),
            Cell::from(project),
            Cell::from(format_time(session.created)),
            Cell::from(format_time(session.last_used)),
            Cell::from(format_size(session.size_bytes)),
        ])
    });
    let widths = [
        Constraint::Length(12),
        Constraint::Percentage(25),
        Constraint::Percentage(32),
        Constraint::Length(12),
        Constraint::Length(12),
        Constraint::Length(10),
    ];
    let table = Table::new(rows, widths)
        .header(
            Row::new([
                "Agent",
                "Session",
                "Project",
                "Created",
                "Last used",
                "Size",
            ])
            .style(Style::default().fg(CYAN).add_modifier(Modifier::BOLD)),
        )
        .block(
            Block::default()
                .title(format!(" Sessions · {} matches ", indices.len()))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(BLUE)),
        )
        .row_highlight_style(Style::default().bg(Color::DarkGray).fg(CYAN));
    let mut state = TableState::default();
    state
        .select((!indices.is_empty()).then_some(app.selected.min(indices.len().saturating_sub(1))));
    frame.render_stateful_widget(table, layout[1], &mut state);
    let selected_id = indices.get(app.selected).map_or_else(String::new, |index| {
        format!(" · ID: {}", app.sessions[*index].id)
    });
    let footer = if app.searching() {
        format!(" Search: {}█", app.filter)
    } else {
        format!(
            " 1-7 agent · 0 all · ↑↓/wheel · Enter resume · f fork · y/Y YOLO · r rename · d trash · / search · ? help · q quit{selected_id}"
        )
    };
    frame.render_widget(
        Paragraph::new(footer).style(Style::default().fg(Color::Black).bg(CYAN)),
        layout[2],
    );
    if app.confirming_delete() {
        modal(
            frame,
            area,
            "Move session to Trash?",
            "Press y to confirm · any other key cancels",
        );
    }
    if app.renaming() {
        modal(
            frame,
            area,
            "Rename session",
            &format!("{}█  (Enter saves · Esc cancels)", app.rename_buffer()),
        );
    }
}

fn format_time(time: Option<std::time::SystemTime>) -> String {
    time.and_then(|value| value.duration_since(std::time::UNIX_EPOCH).ok())
        .map_or_else(|| "—".to_owned(), |value| value.as_secs().to_string())
}

#[allow(clippy::cast_precision_loss)]
fn format_size(size: u64) -> String {
    if size < 1024 {
        return format!("{size} B");
    }
    if size < 1024 * 1024 {
        return format!("{:.1} KB", size as f64 / 1024.0);
    }
    format!("{:.1} MB", size as f64 / (1024.0 * 1024.0))
}

fn modal(frame: &mut Frame, area: Rect, title: &str, body: &str) {
    let popup = centered(Rect::new(area.x, area.y, area.width, area.height), 60, 20);
    frame.render_widget(Clear, popup);
    frame.render_widget(
        Paragraph::new(body).wrap(Wrap { trim: true }).block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(CYAN)),
        ),
        popup,
    );
}

fn centered(area: Rect, width: u16, height: u16) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - height) / 2),
            Constraint::Percentage(height),
            Constraint::Percentage((100 - height) / 2),
        ])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - width) / 2),
            Constraint::Percentage(width),
            Constraint::Percentage((100 - width) / 2),
        ])
        .split(vertical[1])[1]
}
