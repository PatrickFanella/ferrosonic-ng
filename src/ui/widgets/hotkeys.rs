use ratatui::{
    layout::{Alignment, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::app::state::{AppState, Page};
use crate::ui::theme::ThemeColors;

pub fn render_hotkeys_modal(frame: &mut Frame, area: Rect, state: &AppState, colors: ThemeColors) {
    let modal_area = centered_rect(area);

    frame.render_widget(Clear, modal_area);

    let block = Block::default()
        .title("Hotkeys")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(colors.border_focused))
        .style(Style::default().bg(colors.highlight_bg));

    let inner = block.inner(modal_area);
    frame.render_widget(block, modal_area);

    let mut lines = vec![
        section_title("Global", colors),
        key_line("q", "Quit", colors),
        key_line("p / Space", "Play/Pause", colors),
        key_line("h", "Previous track", colors),
        key_line("l", "Next track", colors),
        key_line("Ctrl+R", "Refresh data", colors),
        key_line("t", "Cycle theme", colors),
        key_line("F1..F7", "Page navigation", colors),
        key_line("?", "Toggle this menu", colors),
        Line::default(),
        section_title("Volume", colors),
        key_line("-", "Volume down (2%)", colors),
        key_line("+ / =", "Volume up (2%)", colors),
        Line::default(),
        section_title("Current page", colors),
    ];

    lines.extend(page_specific_lines(state.page, colors));

    let content = Paragraph::new(lines)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true })
        .style(Style::default().fg(colors.primary).bg(colors.highlight_bg));
    frame.render_widget(content, inner);
}

fn centered_rect(area: Rect) -> Rect {
    let width = if area.width >= 40 {
        area.width.clamp(40, 90)
    } else {
        area.width
    };
    let height = if area.height >= 16 {
        area.height.clamp(16, 28)
    } else {
        area.height
    };
    let x = area.x + area.width.saturating_sub(width) / 2;
    let y = area.y + area.height.saturating_sub(height) / 2;
    Rect {
        x,
        y,
        width,
        height,
    }
}

fn section_title(title: &'static str, colors: ThemeColors) -> Line<'static> {
    Line::from(vec![Span::styled(
        title,
        Style::default().fg(colors.accent),
    )])
}

fn key_line(key: &'static str, desc: &'static str, colors: ThemeColors) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("{key:<10}"), Style::default().fg(colors.accent)),
        Span::raw(" "),
        Span::styled(desc, Style::default().fg(colors.muted)),
    ])
}

fn page_specific_lines(page: Page, colors: ThemeColors) -> Vec<Line<'static>> {
    match page {
        Page::Browse => vec![
            key_line("Tab", "Switch focus", colors),
            key_line("Enter", "Play selected", colors),
            key_line("/", "Search", colors),
        ],
        Page::Artists => vec![
            key_line("/", "Filter artists", colors),
            key_line("← / →", "Switch pane", colors),
            key_line("Enter", "Expand / play", colors),
        ],
        Page::Queue => vec![
            key_line("Enter", "Play selected", colors),
            key_line("d", "Remove selected", colors),
            key_line("J / K", "Move selected", colors),
        ],
        Page::Playlists => vec![
            key_line("Tab / ← / →", "Switch pane", colors),
            key_line("Enter", "Play / load", colors),
            key_line("e / n", "Queue append/next", colors),
        ],
        Page::Radio => vec![
            key_line("Enter", "Play station", colors),
            key_line("Space", "Play selected / pause", colors),
            key_line("Ctrl+R", "Refresh stations", colors),
        ],
        Page::Server => vec![
            key_line("Tab", "Next field", colors),
            key_line("Enter", "Test / Save", colors),
            key_line("Backspace", "Delete char", colors),
        ],
        Page::Settings => vec![
            key_line("↑ / ↓", "Select setting", colors),
            key_line("← / → / Enter", "Change option", colors),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::Color;

    fn dummy_colors() -> ThemeColors {
        ThemeColors {
            primary: Color::White,
            secondary: Color::DarkGray,
            accent: Color::Cyan,
            artist: Color::White,
            album: Color::White,
            song: Color::White,
            muted: Color::Gray,
            highlight_bg: Color::Black,
            highlight_fg: Color::White,
            success: Color::Green,
            error: Color::Red,
            playing: Color::Yellow,
            played: Color::Gray,
            border_focused: Color::Cyan,
            border_unfocused: Color::DarkGray,
        }
    }

    #[test]
    fn page_specific_lines_includes_radio_rows() {
        let lines = page_specific_lines(Page::Radio, dummy_colors());
        let text = lines
            .into_iter()
            .map(|l| l.to_string())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(text.contains("Play station"));
        assert!(text.contains("Refresh stations"));
    }

    #[test]
    fn centered_rect_stays_within_bounds() {
        let outer = Rect {
            x: 0,
            y: 0,
            width: 50,
            height: 18,
        };
        let inner = centered_rect(outer);
        assert!(inner.width <= outer.width);
        assert!(inner.height <= outer.height);
        assert!(inner.x >= outer.x);
        assert!(inner.y >= outer.y);
    }

    #[test]
    fn centered_rect_stays_within_bounds_small_terminal() {
        // Terminal smaller than the 40×16 minimum thresholds must not produce
        // a modal that exceeds the available area.
        for (w, h) in [(10u16, 8u16), (30, 10), (39, 15)] {
            let outer = Rect {
                x: 0,
                y: 0,
                width: w,
                height: h,
            };
            let inner = centered_rect(outer);
            assert!(
                inner.width <= outer.width,
                "width {w}: modal width {} > outer width {w}",
                inner.width
            );
            assert!(
                inner.height <= outer.height,
                "height {h}: modal height {} > outer height {h}",
                inner.height
            );
            assert!(inner.x >= outer.x);
            assert!(inner.y >= outer.y);
        }
    }
}
