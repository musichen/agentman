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
            Cell::from(session.id.clone()),
        ])
    });
    let widths = [
        Constraint::Length(14),
        Constraint::Percentage(35),
        Constraint::Percentage(40),
        Constraint::Length(14),
    ];
    let table = Table::new(rows, widths)
        .header(
            Row::new(["Agent", "Session", "Project", "ID"])
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
    let footer = if app.searching() {
        format!(" Search: {}█", app.filter)
    } else {
        " ↑↓/wheel navigate · Enter resume · f fork · y/Y YOLO · r rename · d trash · / search · ? help · q quit ".to_owned()
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
