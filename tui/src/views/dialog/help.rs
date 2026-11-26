use {
    crate::theme::THEME,
    ratatui::{
        Frame,
        layout::{Alignment, Constraint::Length, Flex, Layout},
        style::{Style, Stylize},
        text::Line,
        widgets::{Block, Clear, Padding, Paragraph, Wrap},
    },
};

pub fn draw(frame: &mut Frame) {
    let [area] = Layout::horizontal([Length(120)])
        .flex(Flex::Center)
        .areas(frame.area());
    let [area] = Layout::vertical([Length(37)])
        .flex(Flex::Center)
        .areas(area);

    let block = Block::bordered()
        .bg(THEME.surface)
        .fg(THEME.text)
        .padding(Padding::new(2, 2, 1, 1))
        .title("Help")
        .title_alignment(Alignment::Center);

    let inner_area = block.inner(area);
    let [message_area, control_area] = Layout::vertical([Length(30), Length(1)])
        .flex(Flex::SpaceBetween)
        .areas(inner_area);

    let message = vec![
        Line::from("Glues stores notes in local files or a Git repository:"),
        Line::raw(""),
        Line::from("Local".fg(THEME.accent_text).bg(THEME.accent)),
        Line::raw("Notes are stored locally as separate files."),
        Line::raw(
            "This is the default option for users who prefer a simple, file-based approach without any remote synchronization.",
        ),
        Line::raw(""),
        Line::from("Git".fg(THEME.accent_text).bg(THEME.accent)),
        Line::raw("Git storage requires three inputs: `path`, `remote`, and `branch`."),
        Line::raw(
            "The `path` should point to an existing local Git repository, similar to the file storage path.",
        ),
        Line::raw("For example, you can clone a GitHub repository and use that path."),
        Line::raw(
            "The `remote` and `branch` specify the target remote repository and branch for synchronization.",
        ),
        Line::raw(
            "When you modify notes or directories, Glues will automatically sync changes with the specified remote repository.",
        ),
        Line::raw(""),
    ];
    let paragraph = Paragraph::new(message)
        .wrap(Wrap { trim: true })
        .style(Style::default())
        .alignment(Alignment::Left);
    let control = Line::from("Press any key to close".fg(THEME.text_secondary)).centered();

    frame.render_widget(Clear, area);
    frame.render_widget(block, area);
    frame.render_widget(paragraph, message_area);
    frame.render_widget(control, control_area);
}
