use tui::backend::Backend;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::terminal::Frame;
use tui::text::Spans;
use tui::widgets::Clear;
use tui::widgets::{Block, Borders, Paragraph};

pub fn render_popup<B: Backend>(
    frame: &mut Frame<'_, B>,
    title: &str,
    text: Vec<Spans>,
    percent_area: (u16, u16),
) {
    let size = frame.size();
    let block = Block::default().title(title).borders(Borders::ALL);
    let paragraph = Paragraph::new(text.clone()).block(block);
    let area = centered_rect(percent_area.0, percent_area.1, size);

    frame.render_widget(Clear, area); // this clears out the background
    frame.render_widget(paragraph, area);
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}
