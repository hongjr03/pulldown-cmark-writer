use crate::ast::{Block, Inline};
use crate::text::{Line, Region};
use pulldown_cmark::{Alignment as PAlign, CodeBlockKind, HeadingLevel};

use super::inline::append_inline_to_line;
use super::utils::pad_to_width;
// blocks writer doesn't need the custom trait import here

fn render_paragraph(p: &Vec<Inline>) -> Region {
    let mut r = Region::new();
    let mut defs: Vec<(String, String, String)> = Vec::new();
    let mut curr = Line::new();
    for inl in p {
        match inl {
            Inline::SoftBreak => {
                r.push_back_line(curr);
                curr = Line::new();
            }
            Inline::HardBreak => {
                // Represent hard break by ending the current line with two
                // spaces and starting a new line (stay within same paragraph).
                curr.push("  ");
                r.push_back_line(curr);
                curr = Line::new();
            }
            _ => {
                let mut tmp = Line::new();
                if let Some(def) = append_inline_to_line(&mut tmp, inl) {
                    if !defs.iter().any(|d| d.0 == def.0) {
                        defs.push(def);
                    }
                }
                let s = tmp.apply();
                let mut parts = s.split('\n').peekable();
                while let Some(part) = parts.next() {
                    if !part.is_empty() {
                        curr.push(part);
                    }
                    if parts.peek().is_some() {
                        r.push_back_line(curr);
                        curr = Line::new();
                    }
                }
            }
        }
    }
    r.push_back_line(curr);
    if !defs.is_empty() && !r.is_empty() {
        r.push_back_line(Line::from_str(""));
    }
    for (id, dest, title) in defs {
        if title.is_empty() {
            r.push_back_suffix_line(Line::from_str(&format!("[{}]: {}", id, dest)));
        } else {
            r.push_back_suffix_line(Line::from_str(&format!("[{}]: {} \"{}\"", id, dest, title)));
        }
    }
    r
}

fn render_heading(level: &HeadingLevel, content: &Vec<Inline>) -> Region {
    let mut r = Region::new();
    let mut l = Line::new();
    let n = match level {
        HeadingLevel::H1 => 1usize,
        HeadingLevel::H2 => 2usize,
        HeadingLevel::H3 => 3usize,
        HeadingLevel::H4 => 4usize,
        HeadingLevel::H5 => 5usize,
        HeadingLevel::H6 => 6usize,
    };
    l.push(std::iter::repeat('#').take(n).collect::<String>());
    l.push(" ");
    for inl in content {
        append_inline_to_line(&mut l, inl);
    }
    r.push_back_line(l);
    r
}

fn render_codeblock(kind: &CodeBlockKind<'static>, content: &Region) -> Region {
    let mut r = Region::new();
    match kind {
        CodeBlockKind::Fenced(s) => {
            let lang = s.as_ref();
            let content_str = content.apply();
            let mut max_ticks = 0usize;
            let mut cur = 0usize;
            for ch in content_str.chars() {
                if ch == '`' {
                    cur += 1;
                    if cur > max_ticks {
                        max_ticks = cur;
                    }
                } else {
                    cur = 0;
                }
            }
            let ticks = std::cmp::max(3, max_ticks + 1);
            let fence = "`".repeat(ticks) + lang;
            r.push_back_line(Line::from_str(&fence));
            for l in content_str.lines() {
                r.push_back_line(Line::from_str(l));
            }
            r.push_back_line(Line::from_str(&"`".repeat(ticks)));
        }
        CodeBlockKind::Indented => {
            let content_str = content.apply();
            let mut inner = Region::from_str(&content_str);
            inner.indent_each_line(4);
            for l in inner.into_lines() {
                r.push_back_line(l);
            }
        }
    }
    r
}

fn render_blockquote(children: &Vec<Block>) -> Region {
    let mut inner = Region::new();
    let mut first = true;
    for b in children {
        if !first {
            inner.push_back_line(Line::from_str(""));
        }
        first = false;
        let br = block_to_region(b);
        for l in br.into_lines() {
            inner.push_back_line(l);
        }
    }
    if inner.is_empty() {
        return Region::new();
    }
    inner.prefix_each_line("> ");
    inner
}

fn render_list(ordered: bool, start: Option<u64>, items: &Vec<Vec<Block>>) -> Region {
    let mut r = Region::new();
    for (i, item) in items.iter().enumerate() {
        let marker = if ordered {
            let n = start.unwrap_or(1) + (i as u64);
            format!("{}. ", n)
        } else {
            "- ".to_string()
        };

        // merge consecutive paragraphs inside the item
        let mut merged: Vec<Block> = Vec::new();
        for ch in item {
            if let Some(Block::Paragraph(prev)) = merged.last_mut() {
                match ch {
                    Block::Paragraph(inls) => {
                        prev.extend(inls.clone());
                        continue;
                    }
                    _ => {}
                }
            }
            merged.push(ch.clone());
        }

        let mut item_region = Region::new();
        let mut first = true;
        for ch in &merged {
            if !first {
                item_region.push_back_line(Line::from_str(""));
            }
            first = false;
            let br = block_to_region(ch);
            for l in br.into_lines() {
                item_region.push_back_line(l);
            }
        }

        if item_region.is_empty() {
            // if first block is nested list, skip placeholder
            let mut first_is_list = false;
            if let Some(first_block) = item.get(0) {
                if let Block::List { .. } = first_block {
                    first_is_list = true;
                }
            }
            if !first_is_list {
                item_region.push_back_line(Line::from_str(""));
            }
        }

        item_region.prefix_first_then_indent_rest(marker.as_str());
        for l in item_region.into_lines() {
            r.push_back_line(l);
        }
        r.push_back_line(Line::from_str(""));
    }
    r
}

fn render_rule() -> Region {
    let mut r = Region::new();
    r.push_back_line(Line::from_str("---"));
    r
}

fn render_footnote_def(id: &str, children: &Vec<Block>) -> Region {
    let mut r = Region::new();
    let mut inner = Region::new();
    let mut first = true;
    for b in children {
        if !first {
            inner.push_back_line(Line::from_str(""));
        }
        first = false;
        let br = block_to_region(b);
        for l in br.into_lines() {
            inner.push_back_line(l);
        }
    }
    inner.indent_each_line(4);
    let lines = inner.into_lines();
    if let Some(l0) = lines.get(0) {
        let mut head = Line::from_str(&format!("[^{}]: ", id));
        head.push(l0.apply());
        r.push_back_line(head);
    }
    for ln in lines.iter().skip(1) {
        r.push_back_line(ln.clone());
    }
    r
}

fn cell_to_lines(cell: &Vec<Inline>) -> Vec<String> {
    let mut l = Line::new();
    for inl in cell {
        append_inline_to_line(&mut l, inl);
    }
    l.apply().split('\n').map(|s| s.to_string()).collect()
}

fn render_table_full(aligns: &Vec<PAlign>, rows: &Vec<Vec<Vec<Inline>>>) -> Region {
    let cols = aligns
        .len()
        .max(rows.iter().map(|r| r.len()).max().unwrap_or(0));

    // build cells_text[row_idx][col_idx] -> Vec<String>
    let mut cells_text: Vec<Vec<Vec<String>>> = Vec::new();
    for r in rows {
        let mut row_cells: Vec<Vec<String>> = Vec::new();
        for c in 0..cols {
            if let Some(cell) = r.get(c) {
                row_cells.push(cell_to_lines(cell));
            } else {
                row_cells.push(vec![String::new()]);
            }
        }
        cells_text.push(row_cells);
    }

    let mut col_widths = vec![0usize; cols];
    for row in &cells_text {
        for (ci, cell_lines) in row.iter().enumerate() {
            for line in cell_lines {
                col_widths[ci] =
                    col_widths[ci].max(unicode_width::UnicodeWidthStr::width(line.as_str()));
            }
        }
    }

    let mut reg = Region::new();
    if !cells_text.is_empty() {
        // header is first row
        let header = &cells_text[0];
        let mut header_line = Line::new();
        for c in 0..cols {
            if c > 0 {
                header_line.push(" | ");
            }
            let h = header[c].join("\n");
            header_line.push(pad_to_width(&h, col_widths[c], aligns.get(c)));
        }
        reg.push_back_line(header_line);

        // separator
        let mut sep = Line::new();
        for c in 0..cols {
            if c > 0 {
                sep.push(" | ");
            }
            match aligns.get(c) {
                Some(PAlign::Left) => {
                    sep.push(pad_to_width(
                        &format!(":{}", "-".repeat(col_widths[c].saturating_sub(1))),
                        col_widths[c],
                        None,
                    ));
                }
                Some(PAlign::Right) => {
                    sep.push(pad_to_width(
                        &format!("{}:", "-".repeat(col_widths[c].saturating_sub(1))),
                        col_widths[c],
                        None,
                    ));
                }
                Some(PAlign::Center) => {
                    sep.push(pad_to_width(
                        &format!(":{}:", "-".repeat(col_widths[c].saturating_sub(2))),
                        col_widths[c],
                        None,
                    ));
                }
                _ => {
                    sep.push("-".repeat(col_widths[c]));
                }
            };
        }
        reg.push_back_line(sep);

        // body rows (skip header at idx 0)
        for r_idx in 1..cells_text.len() {
            let mut line = Line::new();
            for c in 0..cols {
                if c > 0 {
                    line.push(" | ");
                }
                let cell_text = cells_text[r_idx][c].join("\n");
                line.push(pad_to_width(&cell_text, col_widths[c], aligns.get(c)));
            }
            reg.push_back_line(line);
        }
    }

    reg
}

pub fn block_to_region(b: &Block) -> Region {
    match b {
        Block::Paragraph(inls) => render_paragraph(inls),
        Block::Heading {
            level, children, ..
        } => render_heading(level, children),
        Block::CodeBlock { kind, content } => render_codeblock(kind, content),
        Block::HtmlBlock(rgn) => {
            let mut r = Region::new();
            for l in rgn.apply().split('\n') {
                r.push_back_line(Line::from_str(l));
            }
            r
        }
        Block::BlockQuote(children) => render_blockquote(children),
        Block::List { start, items } => render_list(start.is_some(), *start, items),
        Block::Rule => render_rule(),
        Block::FootnoteDefinition(id, children) => render_footnote_def(id, children),
        Block::TableFull(aligns, rows) => render_table_full(aligns, rows),
        Block::Custom(c) => {
            // Flatten custom block events into lines: collect Text/Html events
            let mut r = Region::new();
            for ev in c.to_events() {
                match ev {
                    pulldown_cmark::Event::Text(t) | pulldown_cmark::Event::Html(t) => {
                        for l in t.into_string().lines() {
                            r.push_back_line(Line::from_str(l));
                        }
                    }
                    pulldown_cmark::Event::Start(_) | pulldown_cmark::Event::End(_) => {
                        // ignore structural tags when flattening
                    }
                    _ => {}
                }
            }
            r
        }
        _ => Region::new(),
    }
}

pub fn blocks_to_markdown(blocks: &[Block]) -> String {
    let mut out = String::new();
    let mut first = true;
    for b in blocks {
        if !first {
            out.push_str("\n\n");
        }
        first = false;
        let r = block_to_region(b);
        for ln in r.into_lines() {
            out.push_str(&ln.apply());
            out.push('\n');
        }
    }
    out
}
