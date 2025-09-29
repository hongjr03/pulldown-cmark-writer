use crate::ast::block::Block;
use crate::ast::inline::Inline;
use crate::text::{Line, Region};
use pulldown_cmark::{Event, Tag};

/// Convert a pulldown-cmark `Event` slice into a vector of `Block` AST nodes.
/// This is a best-effort parser that understands common tags and will
/// conservatively wrap unknown structures.
/// Parse events into blocks, allowing the caller to pass an optional hook.
///
/// The hook, if provided, is invoked with the remaining slice of events at
/// the current parse position. If the hook recognizes a custom node it may
/// return `Some((consumed, Block))` to indicate it consumed `consumed`
/// events and produced the provided `Block`. The parser will then skip the
/// consumed events and continue. The hook is called before processing the
/// next event and applies at the current nesting level.
// ParseContext is defined and re-exported from `crate::ast::ParseContext`.

pub fn parse_events_to_blocks_with_hook<'a>(
    events: &[Event<'a>],
    mut hook: Option<&mut dyn for<'b> FnMut(&'b [Event<'b>], usize, &crate::ast::ParseContext) -> Option<(usize, Block)>>,
) -> Vec<Block> {
    // A simple stack frame used while parsing Start/End pairs.
    struct Frame<'a> {
        tag: Tag<'a>,
        // collecting inlines vs blocks
        inlines: Vec<Inline>,
        blocks: Vec<Block>,
        collect_inlines: bool,
    }

    fn region_from_cow(s: &str) -> Region {
        Region::from_str(s)
    }

    let mut stack: Vec<Frame> = Vec::new();
    let mut out: Vec<Block> = Vec::new();

    // helper to convert Tag<'a> -> Tag<'static>
    fn tag_to_static(t: &Tag) -> Tag<'static> {
        t.clone().into_static()
    }

    let mut i: usize = 0;
    while i < events.len() {
        // build minimal context for the hook and try it first
        let ctx = crate::ast::ParseContext {
            depth: stack.len(),
            parent_tag: stack.last().map(|f| tag_to_static(&f.tag)),
            parent_collects_inlines: stack.last().map(|f| f.collect_inlines).unwrap_or(false),
            event_index: i,
        };
        if let Some(h) = hook.as_mut() {
            if let Some((consumed, blk)) = h(&events[i..], i, &ctx) {
                out.push(blk);
                i = i.saturating_add(consumed);
                continue;
            }
        }

        let ev = &events[i];
        match ev {
            Event::Start(tag) => {
                // decide whether this tag should collect inlines or blocks
                let collect_inlines = matches!(
                    tag,
                    Tag::Paragraph
                        | Tag::Heading { .. }
                        | Tag::Emphasis
                        | Tag::Strong
                        | Tag::Strikethrough
                        | Tag::Subscript
                        | Tag::Superscript
                        | Tag::Link { .. }
                        | Tag::Image { .. }
                        | Tag::TableCell
                );
                stack.push(Frame {
                    tag: tag.clone(),
                    inlines: Vec::new(),
                    blocks: Vec::new(),
                    collect_inlines,
                });
                i += 1;
            }
            Event::End(_tagend) => {
                if let Some(frame) = stack.pop() {
                    // convert frame into either Block or Inline and append to parent or root
                    use pulldown_cmark::Tag::*;
                    let mut maybe_inline: Option<Inline> = None;
                    let node = match frame.tag {
                        Paragraph => Block::Paragraph(frame.inlines),
                        Heading {
                            level,
                            id,
                            classes,
                            attrs,
                        } => Block::Heading {
                            level,
                            id: id.map(|c| c.to_string()),
                            classes: classes.into_iter().map(|c| c.to_string()).collect(),
                            attrs: attrs
                                .into_iter()
                                .map(|(a, v)| (a.to_string(), v.map(|s| s.to_string())))
                                .collect(),
                            children: frame.inlines,
                        },
                        BlockQuote(_kind) => Block::BlockQuote(frame.blocks),
                        CodeBlock(kind) => {
                            // code block content: concatenate paragraph texts as emitted
                            let mut combined = String::new();
                            for b in frame.blocks.into_iter() {
                                if let Block::Paragraph(inls) = b {
                                    for inl in inls {
                                        if let Inline::Text(r) = inl {
                                            combined.push_str(&r.apply());
                                        }
                                    }
                                }
                            }
                            let content = Region::from_str(&combined);
                            let kind_owned = kind.into_static();
                            Block::CodeBlock {
                                kind: kind_owned,
                                content,
                            }
                        }
                        HtmlBlock => {
                            // build content from any Html blocks collected in frame.blocks
                            // and any inline Html/Text collected in frame.inlines.
                            let mut content = Region::new();
                            for b in frame.blocks.into_iter() {
                                match b {
                                    Block::HtmlBlock(rgn) => {
                                        for l in rgn.apply().split('\n') {
                                            content.push_back_line(Line::from_str(l));
                                        }
                                    }
                                    Block::Paragraph(inls) => {
                                        for inl in inls {
                                            if let Inline::Text(r) = inl {
                                                content.push_back_line(Line::from_str(&r.apply()));
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            for inl in frame.inlines.into_iter() {
                                match inl {
                                    Inline::Text(r) => {
                                        content.push_back_line(Line::from_str(&r.apply()));
                                    }
                                    Inline::Html(r) => {
                                        for l in r.apply().split('\n') {
                                            content.push_back_line(Line::from_str(l));
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            Block::HtmlBlock(content)
                        }
                        List(start) => {
                            let mut items: Vec<Vec<Block>> = Vec::new();
                            for b in frame.blocks.into_iter() {
                                match b {
                                    Block::Item(children) => items.push(children),
                                    other => items.push(vec![other]),
                                }
                            }
                            Block::List { start, items }
                        }
                        Item => Block::Item(frame.blocks),
                        FootnoteDefinition(label) => {
                            Block::FootnoteDefinition(label.to_string(), frame.blocks)
                        }
                        Table(aligns) => {
                            let mut rows: Vec<Vec<Vec<Inline>>> = Vec::new();
                            for b in frame.blocks.into_iter() {
                                match b {
                                    Block::TableRow(cells) => rows.push(cells),
                                    Block::Paragraph(inls) => rows.push(vec![inls]),
                                    other => {
                                        match other {
                                            Block::Item(children) => {
                                                let mut inls_acc: Vec<Inline> = Vec::new();
                                                for ch in children {
                                                    if let Block::Paragraph(mut p_inls) = ch {
                                                        inls_acc.append(&mut p_inls);
                                                    }
                                                }
                                                rows.push(vec![inls_acc]);
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                            }
                            Block::TableFull(aligns, rows)
                        }
                        TableHead | TableRow => {
                            let mut row_cells: Vec<Vec<Inline>> = Vec::new();
                            for b in frame.blocks.into_iter() {
                                match b {
                                    Block::Paragraph(inls) => row_cells.push(inls),
                                    _ => {}
                                }
                            }
                            Block::TableRow(row_cells)
                        }
                        TableCell => Block::Paragraph(frame.inlines),
                        Emphasis => {
                            maybe_inline = Some(Inline::Emphasis(frame.inlines));
                            Block::Paragraph(Vec::new())
                        }
                        Strong => {
                            maybe_inline = Some(Inline::Strong(frame.inlines));
                            Block::Paragraph(Vec::new())
                        }
                        Strikethrough => {
                            maybe_inline = Some(Inline::Strikethrough(frame.inlines));
                            Block::Paragraph(Vec::new())
                        }
                        Subscript => {
                            maybe_inline = Some(Inline::Subscript(frame.inlines));
                            Block::Paragraph(Vec::new())
                        }
                        Superscript => {
                            maybe_inline = Some(Inline::Superscript(frame.inlines));
                            Block::Paragraph(Vec::new())
                        }
                        Tag::Link {
                            link_type,
                            dest_url,
                            title,
                            id,
                        } => {
                            maybe_inline = Some(Inline::Link {
                                link_type,
                                dest: dest_url.to_string(),
                                title: title.to_string(),
                                id: id.to_string(),
                                children: frame.inlines,
                            });
                            Block::Paragraph(Vec::new())
                        }
                        Tag::Image {
                            link_type,
                            dest_url,
                            title,
                            id,
                        } => {
                            maybe_inline = Some(Inline::Image {
                                link_type,
                                dest: dest_url.to_string(),
                                title: title.to_string(),
                                id: id.to_string(),
                                children: frame.inlines,
                            });
                            Block::Paragraph(Vec::new())
                        }
                        Tag::MetadataBlock(_kind) => Block::Paragraph(frame.inlines),
                        _ => Block::Paragraph(frame.inlines),
                    };

                    if let Some(parent) = stack.last_mut() {
                        if let Some(inl) = maybe_inline.take() {
                            if parent.collect_inlines {
                                parent.inlines.push(inl);
                            } else {
                                if let Some(last) = parent.blocks.last_mut() {
                                    match last {
                                        Block::Paragraph(inls) => inls.push(inl),
                                        _ => parent.blocks.push(Block::Paragraph(vec![inl])),
                                    }
                                } else {
                                    parent.blocks.push(Block::Paragraph(vec![inl]));
                                }
                            }
                        } else if parent.collect_inlines {
                            match node {
                                Block::Paragraph(inls) => parent.inlines.extend(inls),
                                other => parent.blocks.push(other),
                            }
                        } else {
                            parent.blocks.push(node);
                        }
                    } else {
                        if let Some(inl) = maybe_inline.take() {
                            out.push(Block::Paragraph(vec![inl]));
                        } else {
                            out.push(node);
                        }
                    }
                }
                i += 1;
            }
            Event::Text(t) => {
                let r = region_from_cow(t);
                if let Some(top) = stack.last_mut() {
                    if top.collect_inlines {
                        top.inlines.push(Inline::Text(r));
                    } else {
                        top.blocks.push(Block::Paragraph(vec![Inline::Text(r)]));
                    }
                } else {
                    out.push(Block::Paragraph(vec![Inline::Text(r)]));
                }
                i += 1;
            }
            Event::Code(t) => {
                let r = region_from_cow(t);
                if let Some(top) = stack.last_mut() {
                    if top.collect_inlines {
                        top.inlines.push(Inline::Code(r));
                    } else {
                        top.blocks.push(Block::Paragraph(vec![Inline::Code(r)]));
                    }
                } else {
                    out.push(Block::Paragraph(vec![Inline::Code(r)]));
                }
                i += 1;
            }
            Event::InlineHtml(t) => {
                let r = region_from_cow(t);
                if let Some(top) = stack.last_mut() {
                    top.inlines.push(Inline::InlineHtml(r));
                } else {
                    out.push(Block::Paragraph(vec![Inline::InlineHtml(r)]));
                }
                i += 1;
            }
            Event::Html(t) => {
                let r = region_from_cow(t);
                if let Some(top) = stack.last_mut() {
                    if top.collect_inlines {
                        top.inlines.push(Inline::Html(r));
                    } else {
                        top.blocks.push(Block::HtmlBlock(r));
                    }
                } else {
                    out.push(Block::HtmlBlock(r));
                }
                i += 1;
            }
            Event::SoftBreak => {
                if let Some(top) = stack.last_mut() {
                    if top.collect_inlines {
                        top.inlines.push(Inline::SoftBreak);
                    } else {
                        top.blocks.push(Block::Paragraph(vec![Inline::SoftBreak]));
                    }
                } else {
                    out.push(Block::Paragraph(vec![Inline::SoftBreak]));
                }
                i += 1;
            }
            Event::HardBreak => {
                if let Some(top) = stack.last_mut() {
                    if top.collect_inlines {
                        top.inlines.push(Inline::HardBreak);
                    } else {
                        top.blocks.push(Block::Paragraph(vec![Inline::HardBreak]));
                    }
                } else {
                    out.push(Block::Paragraph(vec![Inline::HardBreak]));
                }
                i += 1;
            }
            Event::Rule => {
                if let Some(top) = stack.last_mut() {
                    top.blocks.push(Block::Rule);
                } else {
                    out.push(Block::Rule);
                }
                i += 1;
            }
            Event::TaskListMarker(b) => {
                if let Some(top) = stack.last_mut() {
                    top.inlines.push(Inline::Text(Region::from_str(if *b { "[x]" } else { "[ ]" })));
                } else {
                    out.push(Block::Paragraph(vec![Inline::Text(Region::from_str(if *b { "[x]" } else { "[ ]" }))]));
                }
                i += 1;
            }
            Event::FootnoteReference(t) => {
                let s = t.to_string();
                if let Some(top) = stack.last_mut() {
                    top.inlines.push(Inline::FootnoteReference(s));
                } else {
                    out.push(Block::Paragraph(vec![Inline::FootnoteReference(s)]));
                }
                i += 1;
            }
            Event::InlineMath(t) => {
                let r = region_from_cow(t);
                if let Some(top) = stack.last_mut() {
                    top.inlines.push(Inline::InlineMath(r));
                } else {
                    out.push(Block::Paragraph(vec![Inline::InlineMath(r)]));
                }
                i += 1;
            }
            Event::DisplayMath(t) => {
                let r = region_from_cow(t);
                if let Some(top) = stack.last_mut() {
                    top.inlines.push(Inline::DisplayMath(r));
                } else {
                    out.push(Block::Paragraph(vec![Inline::DisplayMath(r)]));
                }
                i += 1;
            }
        }
    }

    out
}

// Backwards compatible wrapper without hook
pub fn parse_events_to_blocks<'a>(events: &[Event<'a>]) -> Vec<Block> {
    parse_events_to_blocks_with_hook(events, None)
}

/// Helper that accepts a list of boxed `BlockParser` trait objects and runs
/// them as parsers by adapting them to the hook signature.
pub fn parse_events_to_blocks_with_parsers<'a>(
    events: &[Event<'a>],
    parsers: &[&dyn crate::ast::custom::BlockParser],
) -> Vec<Block> {
    let mut hook = |evs: &[Event], i: usize, ctx: &crate::ast::ParseContext| -> Option<(usize, Block)> {
        for p in parsers.iter() {
            if let Some((consumed, blk)) = p.try_parse(evs, i, ctx) {
                return Some((consumed, blk));
            }
        }
        None
    };
    parse_events_to_blocks_with_hook(events, Some(&mut hook))
}
 
