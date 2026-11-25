use {
    crate::theme::THEME,
    pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd},
    ratatui::{
        style::{Modifier, Style},
        text::{Line, Span, Text},
    },
};

pub(super) fn render(markdown: &str) -> Text<'static> {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_GFM);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_SMART_PUNCTUATION);
    options.insert(Options::ENABLE_HEADING_ATTRIBUTES);
    options.insert(Options::ENABLE_YAML_STYLE_METADATA_BLOCKS);
    options.insert(Options::ENABLE_PLUSES_DELIMITED_METADATA_BLOCKS);
    options.insert(Options::ENABLE_MATH);
    options.insert(Options::ENABLE_DEFINITION_LIST);
    options.insert(Options::ENABLE_SUPERSCRIPT);
    options.insert(Options::ENABLE_SUBSCRIPT);
    options.insert(Options::ENABLE_WIKILINKS);

    let mut lines = Vec::new();
    let mut current = Vec::new();
    let mut styles = vec![Style::default().fg(THEME.text)];
    let mut lists: Vec<Option<u64>> = Vec::new();
    let mut table_headers: Vec<Vec<Span<'static>>> = Vec::new();
    let mut table_row: Vec<Vec<Span<'static>>> = Vec::new();
    let mut blockquote_depth = 0;
    let flush = |lines: &mut Vec<Line<'static>>, current: &mut Vec<Span<'static>>| {
        if current.is_empty() {
            return;
        }
        lines.push(Line::from(std::mem::take(current)));
    };
    let normalized = normalize_split_table_header(markdown);
    let mut events = Parser::new_ext(&normalized, options).peekable();
    while let Some(event) = events.next() {
        match event {
            Event::Start(tag) => match tag {
                Tag::Table(_) => {
                    flush(&mut lines, &mut current);
                    table_headers.clear();
                    table_row.clear();
                }
                Tag::Heading { level, .. } => styles.push(heading_style(level)),
                Tag::BlockQuote(_) => {
                    blockquote_depth += 1;
                    styles.push(
                        Style::default()
                            .fg(THEME.text_secondary)
                            .add_modifier(Modifier::ITALIC),
                    );
                }
                Tag::List(start) => lists.push(start),
                Tag::Item => {
                    let is_task = matches!(events.peek(), Some(Event::TaskListMarker(_)));
                    if !is_task {
                        let marker = match lists.last_mut() {
                            Some(Some(next)) => {
                                let marker = format!("{next}. ");
                                *next += 1;
                                marker
                            }
                            _ => "• ".to_owned(),
                        };
                        current.push(Span::styled(marker, Style::default().fg(THEME.accent)));
                    }
                }
                Tag::CodeBlock(_) => {
                    styles.push(Style::default().fg(THEME.text_secondary).bg(THEME.surface));
                }
                Tag::MetadataBlock(_) => {
                    flush(&mut lines, &mut current);
                    styles.push(Style::default().fg(THEME.text_secondary).bg(THEME.surface));
                }
                Tag::FootnoteDefinition(label) => {
                    flush(&mut lines, &mut current);
                    current.push(Span::styled(
                        format!("[{}]: ", label),
                        Style::default()
                            .fg(THEME.accent_text)
                            .add_modifier(Modifier::BOLD),
                    ));
                }
                Tag::Emphasis => styles.push(current_style(&styles).add_modifier(Modifier::ITALIC)),
                Tag::Strong => styles.push(current_style(&styles).add_modifier(Modifier::BOLD)),
                Tag::Strikethrough => {
                    styles.push(current_style(&styles).add_modifier(Modifier::CROSSED_OUT));
                }
                Tag::Superscript => {
                    current.push(Span::styled("^{", Style::default().fg(THEME.accent)));
                    styles.push(current_style(&styles).add_modifier(Modifier::DIM));
                }
                Tag::Subscript => {
                    current.push(Span::styled("_{", Style::default().fg(THEME.accent)));
                    styles.push(current_style(&styles).add_modifier(Modifier::DIM));
                }
                Tag::Link { .. } => styles.push(
                    current_style(&styles)
                        .fg(THEME.accent_text)
                        .add_modifier(Modifier::UNDERLINED),
                ),
                Tag::Image { .. } => {
                    current.push(Span::styled("▧ ", Style::default().fg(THEME.accent)));
                }
                Tag::TableHead => {
                    if !current.is_empty() {
                        flush(&mut lines, &mut current);
                    }
                    table_row.clear();
                }
                Tag::TableRow => table_row.clear(),
                Tag::TableCell => current.clear(),
                Tag::DefinitionListTitle => {
                    styles.push(current_style(&styles).add_modifier(Modifier::BOLD));
                }
                Tag::DefinitionListDefinition => {
                    current.push(Span::styled("  • ", Style::default().fg(THEME.accent)));
                }
                Tag::Paragraph if blockquote_depth > 0 => {
                    current.push(Span::styled(
                        "│ ".repeat(blockquote_depth),
                        Style::default().fg(THEME.accent),
                    ));
                }
                _ => {}
            },
            Event::End(tag) => match tag {
                TagEnd::TableCell => table_row.push(std::mem::take(&mut current)),
                TagEnd::Paragraph => {
                    flush(&mut lines, &mut current);
                    if lists.is_empty() {
                        blank_line(&mut lines);
                    }
                }
                TagEnd::Heading(_) => {
                    styles.pop();
                    flush(&mut lines, &mut current);
                    blank_line(&mut lines);
                }
                TagEnd::Item => flush(&mut lines, &mut current),
                TagEnd::List(_) => {
                    lists.pop();
                    flush(&mut lines, &mut current);
                    blank_line(&mut lines);
                }
                TagEnd::BlockQuote(_) => {
                    blockquote_depth = blockquote_depth.saturating_sub(1);
                    styles.pop();
                    flush(&mut lines, &mut current);
                    blank_line(&mut lines);
                }
                TagEnd::CodeBlock => {
                    styles.pop();
                    flush(&mut lines, &mut current);
                    blank_line(&mut lines);
                }
                TagEnd::MetadataBlock(_) => {
                    styles.pop();
                    flush(&mut lines, &mut current);
                    blank_line(&mut lines);
                }
                TagEnd::FootnoteDefinition => {
                    flush(&mut lines, &mut current);
                    blank_line(&mut lines);
                }
                TagEnd::Emphasis | TagEnd::Strong | TagEnd::Strikethrough | TagEnd::Link => {
                    styles.pop();
                }
                TagEnd::Superscript => {
                    styles.pop();
                    current.push(Span::styled("}", Style::default().fg(THEME.accent)));
                }
                TagEnd::Subscript => {
                    styles.pop();
                    current.push(Span::styled("}", Style::default().fg(THEME.accent)));
                }
                TagEnd::DefinitionListTitle => {
                    styles.pop();
                    flush(&mut lines, &mut current);
                }
                TagEnd::DefinitionListDefinition => {
                    flush(&mut lines, &mut current);
                    blank_line(&mut lines);
                }
                TagEnd::DefinitionList => blank_line(&mut lines),
                TagEnd::TableHead => {
                    table_headers = std::mem::take(&mut table_row);
                }
                TagEnd::TableRow => {
                    render_table_row(&mut lines, &table_headers, std::mem::take(&mut table_row));
                }
                TagEnd::Table => {
                    flush(&mut lines, &mut current);
                    blank_line(&mut lines);
                }
                _ => {}
            },
            Event::Text(text) => {
                append_text(&mut lines, &mut current, &text, current_style(&styles))
            }
            Event::Code(code) => current.push(Span::styled(
                code.into_string(),
                Style::default().fg(THEME.accent_text).bg(THEME.surface),
            )),
            Event::InlineMath(math) | Event::DisplayMath(math) => current.push(Span::styled(
                math.into_string(),
                Style::default().fg(THEME.accent_text).bg(THEME.surface),
            )),
            Event::Html(html) | Event::InlineHtml(html) => {
                append_text(&mut lines, &mut current, &html, current_style(&styles));
            }
            Event::SoftBreak => current.push(Span::raw(" ")),
            Event::HardBreak => flush(&mut lines, &mut current),
            Event::Rule => {
                flush(&mut lines, &mut current);
                lines.push(Line::from(Span::styled(
                    "────────────────────────────────",
                    Style::default().fg(THEME.inactive_text),
                )));
                blank_line(&mut lines);
            }
            Event::TaskListMarker(checked) => current.push(Span::styled(
                if checked { "☑ " } else { "☐ " },
                Style::default().fg(THEME.accent),
            )),
            Event::FootnoteReference(name) => {
                current.push(Span::styled(format!("[{}]", name), current_style(&styles)))
            }
        }
    }

    flush(&mut lines, &mut current);
    while lines.last().is_some_and(|line| line.spans.is_empty()) {
        lines.pop();
    }
    Text::from(lines)
}

fn normalize_split_table_header(markdown: &str) -> String {
    let source_lines = markdown.lines().collect::<Vec<_>>();
    let mut output = Vec::with_capacity(source_lines.len());
    let mut index = 0;

    while index < source_lines.len() {
        if index + 2 < source_lines.len() {
            let header = source_lines[index].trim_end();
            let fragment = source_lines[index + 1].trim();
            let last_header_cell = header
                .trim_end_matches('|')
                .rsplit('|')
                .next()
                .unwrap_or_default()
                .trim_end();

            if header.matches('|').count() >= 2
                && header.ends_with('|')
                && fragment.chars().count() <= 2
                && !fragment.is_empty()
                && last_header_cell.ends_with(fragment)
                && is_table_separator(source_lines[index + 2])
            {
                output.push(source_lines[index]);
                index += 2;
                continue;
            }
        }

        output.push(source_lines[index]);
        index += 1;
    }

    output.join("\n")
}

fn is_table_separator(line: &str) -> bool {
    let line = line.trim().trim_matches('|');
    !line.is_empty()
        && line.split('|').all(|cell| {
            let cell = cell.trim().trim_matches(':');
            !cell.is_empty() && cell.chars().all(|character| character == '-')
        })
}

fn blank_line(lines: &mut Vec<Line<'static>>) {
    if !lines.last().is_some_and(|line| line.spans.is_empty()) {
        lines.push(Line::default());
    }
}

fn append_text(
    lines: &mut Vec<Line<'static>>,
    current: &mut Vec<Span<'static>>,
    text: &str,
    style: Style,
) {
    let mut parts = text.split('\n').peekable();
    while let Some(part) = parts.next() {
        if !part.is_empty() {
            current.push(Span::styled(part.to_owned(), style));
        }
        if parts.peek().is_some() {
            if current.is_empty() {
                lines.push(Line::default());
            } else {
                lines.push(Line::from(std::mem::take(current)));
            }
        }
    }
}

fn heading_style(level: HeadingLevel) -> Style {
    let color = match level {
        HeadingLevel::H1 => THEME.accent_text,
        HeadingLevel::H2 => THEME.highlight,
        _ => THEME.success_text,
    };
    Style::default().fg(color).add_modifier(Modifier::BOLD)
}

fn render_table_row(
    lines: &mut Vec<Line<'static>>,
    headers: &[Vec<Span<'static>>],
    cells: Vec<Vec<Span<'static>>>,
) {
    let Some((title, details)) = cells.split_first() else {
        return;
    };
    let mut title_line = vec![Span::styled("◆ ", Style::default().fg(THEME.accent))];
    title_line.extend(title.iter().cloned());
    lines.push(Line::from(title_line));

    for (index, cell) in details.iter().enumerate() {
        let label = headers
            .get(index + 1)
            .map(|header| {
                header
                    .iter()
                    .map(|span| span.content.as_ref())
                    .collect::<String>()
            })
            .filter(|label| !label.trim().is_empty())
            .unwrap_or_else(|| format!("Column {}", index + 2));
        let mut line = vec![
            Span::raw("  "),
            Span::styled(
                format!("{label}: "),
                Style::default()
                    .fg(THEME.accent_text)
                    .add_modifier(Modifier::BOLD),
            ),
        ];
        line.extend(cell.iter().cloned());
        lines.push(Line::from(line));
    }
    blank_line(lines);
}

fn current_style(styles: &[Style]) -> Style {
    styles
        .last()
        .copied()
        .unwrap_or_else(|| Style::default().fg(THEME.text))
}

#[cfg(test)]
mod tests {
    use super::render;

    fn line_text(text: &ratatui::text::Text<'static>) -> Vec<String> {
        text.lines
            .iter()
            .map(|line| {
                line.spans
                    .iter()
                    .map(|span| span.content.as_ref())
                    .collect::<String>()
            })
            .collect()
    }

    #[test]
    fn renders_blocks_and_nested_inline_formatting() {
        let rendered = line_text(&render(
            "# Heading\n\nA **bold _nested_ word** and [link](https://example.com).\n\n> quoted",
        ));
        assert_eq!(rendered[0], "Heading");
        assert_eq!(rendered[2], "A bold nested word and link.");
        assert_eq!(rendered[4], "│ quoted");
    }

    #[test]
    fn splits_fenced_code_into_terminal_rows() {
        let rendered = line_text(&render("```rust\nlet a = 1;\nlet b = 2;\n```"));
        assert_eq!(rendered, ["let a = 1;", "let b = 2;"]);
    }

    #[test]
    fn renders_ordered_and_task_lists_without_extra_bullets() {
        let rendered = line_text(&render("1. first\n2. second\n\n- [x] done\n- [ ] todo"));
        assert_eq!(rendered, ["1. first", "2. second", "", "☑ done", "☐ todo"]);
    }

    #[test]
    fn renders_tables_as_labeled_rows_in_a_narrow_pane() {
        let rendered = line_text(&render(
            "| Programme | Ranking | Assessment |\n|---|---:|---|\n| Alpha Robotics | #73 | Strong research and practical work |\n| Beta Vision | #120 | Focuses on perception |",
        ));
        assert_eq!(
            rendered,
            [
                "◆ Alpha Robotics",
                "  Ranking: #73",
                "  Assessment: Strong research and practical work",
                "",
                "◆ Beta Vision",
                "  Ranking: #120",
                "  Assessment: Focuses on perception",
            ]
        );
    }

    #[test]
    fn repairs_the_split_duplicate_suffix_in_the_saved_chart_header() {
        let rendered = line_text(&render(
            "| Programme | Assessment for you |\nu\n|---|---|\n| Robotics | Good fit |",
        ));
        assert_eq!(rendered, ["◆ Robotics", "  Assessment for you: Good fit"]);
    }

    #[test]
    fn parses_strikethrough_subscript_superscript_and_math() {
        let rendered = line_text(&render(
            "~~old~~ ~subscript~ and ^superscript^ and $y = mx + b$.",
        ));
        assert_eq!(
            rendered,
            ["old _{subscript} and ^{superscript} and y = mx + b."]
        );
    }
}
