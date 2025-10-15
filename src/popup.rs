use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    prelude::style::{Color, Style},
    text::Line,
    widgets::{BarChart, Block, Borders, Clear, Paragraph},
    Frame,
};

pub fn render_popup(frame: &mut Frame<'_>, title: &str, text: &[Line], percent_area: (u16, u16)) {
    let size = frame.area();
    let block = Block::default().title(title).borders(Borders::ALL);
    let style = Style::default().fg(Color::White).bg(Color::Black);
    let paragraph = Paragraph::new(text.to_owned()).block(block).style(style);
    let area = centered_rect(percent_area.0, percent_area.1, size);

    frame.render_widget(Clear, area); // this clears out the background
    frame.render_widget(paragraph, area);
}

pub fn render_bar_chart_popup(frame: &mut Frame<'_>, barchart: BarChart, percent_area: (u16, u16)) {
    let size = frame.area();
    let area = centered_rect(percent_area.0, percent_area.1, size);

    frame.render_widget(Clear, area); // this clears out the background
    frame.render_widget(barchart, area);
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

#[cfg(test)]
mod tests {
    use ratatui::{
        backend::TestBackend, buffer::Buffer, layout::Rect, style::Style, text::Span, Terminal,
    };

    use super::*;
    #[test]
    fn test_centered_rect() {
        let rect = Rect::new(0, 0, 100, 100);
        let new_rect = centered_rect(50, 50, rect);
        let expected_rect = Rect::new(25, 25, 50, 50);
        assert_eq!(new_rect, expected_rect);
    }

    #[test]
    fn test_render_popup() {
        let backend = TestBackend::new(14, 15);
        let mut terminal = Terminal::new(backend).unwrap();
        let text = vec![Line::from(Span::styled("text", Style::default()))];
        terminal
            .draw(|f| {
                render_popup(f, "coso", &text, (50, 50));
            })
            .unwrap();
        let mut expected = Buffer::with_lines(vec![
            "              ",
            "              ",
            "              ",
            "              ",
            "    ┌coso─┐   ",
            "    │text │   ",
            "    │     │   ",
            "    │     │   ",
            "    │     │   ",
            "    │     │   ",
            "    └─────┘   ",
            "              ",
            "              ",
            "              ",
            "              ",
        ]);
        for x in 4..=10 {
            for y in 4..=10 {
                expected[(x, y)].set_fg(Color::White);
                expected[(x, y)].set_bg(Color::Black);
            }
        }

        terminal.backend().assert_buffer(&expected);
    }
}
