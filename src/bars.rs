use ratatui::{
    backend::Backend,
    style::{Color, Style},
    terminal::Frame,
    text::Line,
    widgets::{Bar, BarChart, BarGroup, Block, Borders},
};

use crate::{
    app::App,
    container::Container,
    popup::{centered_rect, render_bar_chart_popup},
};

pub fn render_bar_chart<B: Backend>(frame: &mut Frame<'_, B>, app: &App) {
    let bargroup = create_groups(app);
    let rect = centered_rect(50, 50, frame.size());
    let containers_count = app.containers.len() as u16;
    let bar_width = (rect.width - (containers_count)) / containers_count;
    let corrected_bw = if bar_width * containers_count + containers_count == rect.width {
        bar_width - 1
    } else {
        bar_width
    };
    let title = "Counts";
    let barchart = BarChart::default()
        .block(create_block(title))
        .data(bargroup)
        .bar_gap(1)
        .bar_width(corrected_bw)
        .value_style(Style::default().fg(Color::Black));
    render_bar_chart_popup(frame, barchart, (50, 50));
}

fn create_block(title: &str) -> Block {
    Block::default().title(title).borders(Borders::ALL)
}

fn create_groups<'a>(app: &'a App) -> BarGroup<'a> {
    let bars: Vec<Bar> = app.containers.iter().map(create_bar).collect();
    BarGroup::default().bars(&bars)
}

fn create_bar<'a>(c: &'a Container) -> Bar<'a> {
    Bar::default()
        .value(c.get_count())
        .style(Style::default().fg(c.state.color))
        .value_style(Style::default().fg(c.state.color))
        .label(Line::from(c.text.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_block() {
        let block = create_block("block");
        let block_base = Block::default().title("block").borders(Borders::ALL);
        assert_eq!(block_base, block);
    }

    #[test]
    fn test_create_bar() {
        let container = Container::new("test".to_string(), 1);
        let bar = create_bar(&container);
        let base_bar = Bar::default()
            .value(0)
            .style(Style::default().fg(Color::Red))
            .value_style(Style::default().fg(Color::Red))
            .label(Line::from("test"));

        assert_eq!(base_bar, bar);
    }
}
