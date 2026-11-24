use crate::DocBlock;
use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag, TagEnd};
use slint::StyledText;

const PARAGRAPH: i32 = 0;
const RULE: i32 = 13;
const TABLE: i32 = 14;
const CODE: i32 = 10;
const QUOTE: i32 = 11;
const LIST_ITEM: i32 = 12;

#[derive(Default)]
struct Table {
    rows: Vec<Vec<String>>,
    row: Vec<String>,
    cell: String,
}

struct Item {
    marker: String,
    indent: i32,
}

struct Code {
    language: String,
    body: String,
}

pub fn to_blocks(source: &str) -> Vec<DocBlock> {
    let options = Options::ENABLE_TABLES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_FOOTNOTES;

    let mut blocks = Vec::new();
    let mut inline = String::new();
    let mut plain = String::new();
    let mut heading: Option<u8> = None;
    let mut quote_depth = 0usize;
    let mut lists: Vec<(bool, u64)> = Vec::new();
    let mut item: Option<Item> = None;
    let mut code: Option<Code> = None;
    let mut table: Option<Table> = None;
    let mut links: Vec<String> = Vec::new();
    let mut images: Vec<String> = Vec::new();

    for event in Parser::new_ext(source, options) {
        match event {
            Event::Start(tag) => match tag {
                Tag::Paragraph => {
                    if item.is_some() && !inline.trim().is_empty() {
                        inline.push_str("\n\n");
                        plain.push('\n');
                    }
                }
                Tag::Heading { level, .. } => {
                    flush_item(&mut blocks, &mut inline, &mut plain, item.take());
                    heading = Some(level as u8);
                }
                Tag::BlockQuote(_) => quote_depth += 1,
                Tag::CodeBlock(kind) => {
                    flush_item(&mut blocks, &mut inline, &mut plain, item.take());
                    let language = match kind {
                        CodeBlockKind::Fenced(info) => {
                            info.split_whitespace().next().unwrap_or("").to_string()
                        }
                        CodeBlockKind::Indented => String::new(),
                    };
                    code = Some(Code { language, body: String::new() });
                }
                Tag::List(start) => {
                    flush_item(&mut blocks, &mut inline, &mut plain, item.take());
                    lists.push((start.is_some(), start.unwrap_or(1)));
                }
                Tag::Item => {
                    inline.clear();
                    plain.clear();
                    let depth = lists.len().saturating_sub(1) as i32;
                    let marker = match lists.last_mut() {
                        Some((ordered, next)) => {
                            let marker = if *ordered {
                                format!("{}.", *next)
                            } else {
                                "\u{2022}".to_string()
                            };
                            *next += 1;
                            marker
                        }
                        None => "\u{2022}".to_string(),
                    };
                    item = Some(Item { marker, indent: depth });
                }
                Tag::Table(_) => {
                    flush_item(&mut blocks, &mut inline, &mut plain, item.take());
                    table = Some(Table::default());
                }
                Tag::TableHead | Tag::TableRow => {
                    if let Some(table) = &mut table {
                        table.row.clear();
                    }
                }
                Tag::TableCell => {
                    if let Some(table) = &mut table {
                        table.cell.clear();
                    }
                }
                Tag::Strong if table.is_none() => inline.push_str("**"),
                Tag::Emphasis if table.is_none() => inline.push('*'),
                Tag::Strikethrough if table.is_none() => inline.push_str("~~"),
                Tag::Link { dest_url, .. } if table.is_none() => {
                    links.push(dest_url.to_string());
                    inline.push('[');
                }
                Tag::Image { dest_url, .. } if table.is_none() => {
                    images.push(dest_url.to_string());
                    inline.push('[');
                }
                _ => {}
            },
            Event::End(tag) => match tag {
                TagEnd::Paragraph => {
                    if item.is_none() && heading.is_none() && code.is_none() && table.is_none() {
                        let kind = if quote_depth > 0 { QUOTE } else { PARAGRAPH };
                        flush(&mut blocks, &mut inline, &mut plain, kind, "", 0, 1.0);
                    }
                }
                TagEnd::Heading(_) => {
                    let level = heading.take().unwrap_or(1) as i32;
                    let scale = match level {
                        1 => 1.85,
                        2 => 1.5,
                        3 => 1.25,
                        4 => 1.1,
                        5 => 1.0,
                        _ => 0.9,
                    };
                    flush(
                        &mut blocks,
                        &mut inline,
                        &mut plain,
                        1 + (level - 1).clamp(0, 5),
                        "",
                        0,
                        scale,
                    );
                }
                TagEnd::BlockQuote(_) => quote_depth = quote_depth.saturating_sub(1),
                TagEnd::CodeBlock => {
                    if let Some(code) = code.take() {
                        let body = code.body.trim_end_matches(['\n', '\r']);
                        blocks.push(DocBlock {
                            kind: CODE,
                            content: StyledText::from_plain_text(body),
                            text: body.into(),
                            marker: code.language.into(),
                            indent: 0,
                            scale: 0.85,
                        });
                    }
                }
                TagEnd::List(_) => {
                    lists.pop();
                }
                TagEnd::Item => {
                    flush_item(&mut blocks, &mut inline, &mut plain, item.take());
                }
                TagEnd::TableCell => {
                    if let Some(table) = &mut table {
                        let cell = table.cell.trim().to_string();
                        table.row.push(cell);
                        table.cell.clear();
                    }
                }
                TagEnd::TableHead | TagEnd::TableRow => {
                    if let Some(table) = &mut table
                        && !table.row.is_empty()
                    {
                        table.rows.push(std::mem::take(&mut table.row));
                    }
                }
                TagEnd::Table => {
                    if let Some(table) = table.take() {
                        let text = format_table(&table.rows);
                        if !text.is_empty() {
                            blocks.push(DocBlock {
                                kind: TABLE,
                                content: StyledText::from_plain_text(&text),
                                text: text.into(),
                                marker: "".into(),
                                indent: 0,
                                scale: 0.85,
                            });
                        }
                    }
                }
                TagEnd::Strong if table.is_none() => inline.push_str("**"),
                TagEnd::Emphasis if table.is_none() => inline.push('*'),
                TagEnd::Strikethrough if table.is_none() => inline.push_str("~~"),
                TagEnd::Link if table.is_none() => {
                    if let Some(url) = links.pop() {
                        inline.push_str("](<");
                        inline.push_str(&url);
                        inline.push_str(">)");
                    }
                }
                TagEnd::Image if table.is_none() => {
                    if let Some(url) = images.pop() {
                        inline.push_str("](<");
                        inline.push_str(&url);
                        inline.push_str(">)");
                    }
                }
                _ => {}
            },
            Event::Text(text) => {
                if let Some(code) = &mut code {
                    code.body.push_str(&text);
                } else if let Some(table) = &mut table {
                    table.cell.push_str(&text);
                } else {
                    escape_markdown(&text, &mut inline);
                    plain.push_str(&text);
                }
            }
            Event::Code(text) => {
                if let Some(table) = &mut table {
                    table.cell.push_str(&text);
                } else {
                    push_code_span(&text, &mut inline);
                    plain.push_str(&text);
                }
            }
            Event::SoftBreak => {
                inline.push(' ');
                plain.push(' ');
            }
            Event::HardBreak => {
                inline.push_str("  \n");
                plain.push('\n');
            }
            Event::Rule => {
                flush_item(&mut blocks, &mut inline, &mut plain, item.take());
                blocks.push(DocBlock {
                    kind: RULE,
                    content: StyledText::default(),
                    text: "".into(),
                    marker: "".into(),
                    indent: 0,
                    scale: 1.0,
                });
            }
            Event::TaskListMarker(checked) => {
                let marker = if checked { "[x] " } else { "[ ] " };
                inline.push_str(marker);
                plain.push_str(marker);
            }
            Event::InlineHtml(html) => {
                if let Some(table) = &mut table {
                    table.cell.push_str(&html);
                } else if is_line_break(&html) {
                    inline.push_str("  \n");
                    plain.push('\n');
                } else if is_underline(&html) {
                    inline.push_str(&html);
                }
            }
            Event::Html(_) => {}
            Event::FootnoteReference(name) => {
                let marker = format!("[{}]", name);
                inline.push_str(&marker);
                plain.push_str(&marker);
            }
            Event::InlineMath(math) | Event::DisplayMath(math) => {
                inline.push_str(&math);
                plain.push_str(&math);
            }
        }
    }

    if let Some(item) = item.take() {
        flush_item(&mut blocks, &mut inline, &mut plain, Some(item));
    } else if !plain.trim().is_empty() {
        let kind = if quote_depth > 0 { QUOTE } else { PARAGRAPH };
        flush(&mut blocks, &mut inline, &mut plain, kind, "", 0, 1.0);
    }

    blocks
}

pub fn word_count(source: &str) -> i32 {
    source.split_whitespace().count().min(i32::MAX as usize) as i32
}

fn flush_item(
    blocks: &mut Vec<DocBlock>,
    inline: &mut String,
    plain: &mut String,
    item: Option<Item>,
) {
    if let Some(item) = item {
        flush(
            blocks,
            inline,
            plain,
            LIST_ITEM,
            &item.marker,
            item.indent,
            1.0,
        );
    }
}

fn flush(
    blocks: &mut Vec<DocBlock>,
    inline: &mut String,
    plain: &mut String,
    kind: i32,
    marker: &str,
    indent: i32,
    scale: f32,
) {
    let inline_text = inline.trim();
    let plain_text = plain.trim();

    if inline_text.is_empty() && plain_text.is_empty() {
        inline.clear();
        plain.clear();
        return;
    }

    let content = StyledText::from_markdown(inline_text)
        .unwrap_or_else(|_| StyledText::from_plain_text(plain_text));

    blocks.push(DocBlock {
        kind,
        content,
        text: plain_text.into(),
        marker: marker.into(),
        indent,
        scale,
    });

    inline.clear();
    plain.clear();
}

fn escape_markdown(text: &str, out: &mut String) {
    for character in text.chars() {
        if matches!(
            character,
            '\\' | '`'
                | '*'
                | '_'
                | '{'
                | '}'
                | '['
                | ']'
                | '('
                | ')'
                | '<'
                | '>'
                | '~'
                | '|'
                | '#'
                | '!'
                | '+'
                | '-'
                | '.'
        ) {
            out.push('\\');
        }
        out.push(character);
    }
}

fn push_code_span(code: &str, out: &mut String) {
    let mut ticks = 1;
    while code.contains(&"`".repeat(ticks)) {
        ticks += 1;
    }

    let fence = "`".repeat(ticks);
    let pad = code.starts_with('`') || code.ends_with('`');
    out.push_str(&fence);
    if pad {
        out.push(' ');
    }
    out.push_str(code);
    if pad {
        out.push(' ');
    }
    out.push_str(&fence);
}

fn is_line_break(html: &str) -> bool {
    matches!(
        html.to_ascii_lowercase().as_str(),
        "<br>" | "<br/>" | "<br />"
    )
}

fn is_underline(html: &str) -> bool {
    let html = html.to_ascii_lowercase();
    html == "<u>" || html == "</u>"
}

fn format_table(rows: &[Vec<String>]) -> String {
    let columns = rows.iter().map(Vec::len).max().unwrap_or(0);
    if columns == 0 {
        return String::new();
    }

    let mut widths = vec![1usize; columns];
    for row in rows {
        for (index, cell) in row.iter().enumerate() {
            widths[index] = widths[index].max(cell.chars().count());
        }
    }

    let mut output = String::new();
    for (row_index, row) in rows.iter().enumerate() {
        for column in 0..columns {
            let cell = row.get(column).map(String::as_str).unwrap_or("");
            if column > 0 {
                output.push_str("  ");
            }
            output.push_str(cell);
            if column + 1 < columns {
                let pad = widths[column].saturating_sub(cell.chars().count());
                output.push_str(&" ".repeat(pad));
            }
        }
        output.push('\n');

        if row_index == 0 && rows.len() > 1 {
            for column in 0..columns {
                if column > 0 {
                    output.push_str("  ");
                }
                output.push_str(&"-".repeat(widths[column]));
            }
            output.push('\n');
        }
    }

    output.trim_end().to_string()
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_keeps_header_and_does_not_leak_markup() {
        let blocks = to_blocks("| a | b |\n|---|---|\n| **x** | [l](http://e.com) |\n\nAfter.\n");
        assert_eq!(blocks[0].kind, TABLE);
        assert!(blocks[0].text.starts_with("a  b\n-  -\nx"));
        assert_eq!(blocks[1].text.as_str(), "After.");
        assert!(StyledText::from_markdown(r"After\.").is_ok());
    }

    #[test]
    fn link_with_spaces_parses() {
        assert!(StyledText::from_markdown("[x](<a b/(c).md>)").is_ok());
        assert_eq!(to_blocks("[x](<a b.md>)")[0].text.as_str(), "x");
    }
}
