use crate::ast::block::Block;
use crate::ast::inline::Inline;
use crate::text::{Line, Region};
use pulldown_cmark::{Event, Tag};

/// Convert a pulldown-cmark `Event` slice into a vector of `Block` AST nodes.
/// This is a best-effort parser that understands common tags and will
/// conservatively wrap unknown structures.
pub fn parse_events_to_blocks<'a>(events: &[Event<'a>]) -> Vec<Block> {
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

    for ev in events {
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
            }
            Event::End(_tagend) => {
                if let Some(frame) = stack.pop() {
                    // convert frame into either Block or Inline and append to parent or root
                    use pulldown_cmark::Tag::*;
                    // For span-level tags we prefer to construct Inline nodes so
                    // that emphasis/strong/link/image etc. are preserved.
                    // If the parent collects inlines, push the Inline directly;
                    // otherwise wrap into a Paragraph block as a fallback.
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
                            let mut content = Region::new();
                            for inl in frame.inlines.into_iter() {
                                if let Inline::Text(r) = inl {
                                    content.push_back_line(Line::from_str(&r.apply()));
                                }
                            }
                            Block::HtmlBlock(content)
                        }
                        List(start) => {
                            // every child block is expected to be an Item; collect sequential Items
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
                            // Convert collected row blocks (TableRow) or paragraph rows into a full table
                            let mut rows: Vec<Vec<Vec<Inline>>> = Vec::new();
                            for b in frame.blocks.into_iter() {
                                match b {
                                    Block::TableRow(cells) => rows.push(cells),
                                    Block::Paragraph(inls) => rows.push(vec![inls]),
                                    other => {
                                        // fallback: try to extract paragraph children
                                        match other {
                                            Block::Item(children) => {
                                                // flatten items into a single row cell if possible
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
                            // Table head/row: collect child cell Paragraph blocks into a TableRow
                            let mut row_cells: Vec<Vec<Inline>> = Vec::new();
                            for b in frame.blocks.into_iter() {
                                match b {
                                    Block::Paragraph(inls) => row_cells.push(inls),
                                    _ => {}
                                }
                            }
                            Block::TableRow(row_cells)
                        }
                        TableCell => {
                            // A table cell collects inlines
                            Block::Paragraph(frame.inlines)
                        }
                        // span-level tags become inlines wrapped in a Paragraph when lifted
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
                                // parent collects blocks; if the last block is a
                                // Paragraph, append this inline to it so that
                                // runs of text/inline spans remain in the same
                                // paragraph (important for lists and similar).
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
                            // convert block to inline text if possible
                            match node {
                                Block::Paragraph(inls) => parent.inlines.extend(inls),
                                other => parent.blocks.push(other),
                            }
                        } else {
                            parent.blocks.push(node);
                        }
                    } else {
                        // no parent: if we produced an inline, wrap into paragraph
                        if let Some(inl) = maybe_inline.take() {
                            out.push(Block::Paragraph(vec![inl]));
                        } else {
                            out.push(node);
                        }
                    }
                }
            }
            Event::Text(t) => {
                let r = region_from_cow(t);
                if let Some(top) = stack.last_mut() {
                    if top.collect_inlines {
                        top.inlines.push(Inline::Text(r));
                    } else {
                        // text outside inline container: wrap into paragraph
                        top.blocks.push(Block::Paragraph(vec![Inline::Text(r)]));
                    }
                } else {
                    out.push(Block::Paragraph(vec![Inline::Text(r)]));
                }
            }
            Event::Code(t) => {
                let r = region_from_cow(t);
                if let Some(top) = stack.last_mut() {
                    if top.collect_inlines {
                        top.inlines.push(Inline::Code(r));
                    } else {
                        // not collecting inlines: wrap code as a paragraph block
                        top.blocks.push(Block::Paragraph(vec![Inline::Code(r)]));
                    }
                } else {
                    out.push(Block::Paragraph(vec![Inline::Code(r)]));
                }
            }
            Event::InlineHtml(t) => {
                let r = region_from_cow(t);
                if let Some(top) = stack.last_mut() {
                    top.inlines.push(Inline::InlineHtml(r));
                } else {
                    out.push(Block::Paragraph(vec![Inline::InlineHtml(r)]));
                }
            }
            Event::Html(t) => {
                let r = region_from_cow(t);
                if let Some(top) = stack.last_mut() {
                    // treated as a block if parent collects blocks
                    if top.collect_inlines {
                        top.inlines.push(Inline::Html(r));
                    } else {
                        top.blocks.push(Block::HtmlBlock(r));
                    }
                } else {
                    out.push(Block::HtmlBlock(r));
                }
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
            }
            Event::Rule => {
                if let Some(top) = stack.last_mut() {
                    top.blocks.push(Block::Rule);
                } else {
                    out.push(Block::Rule);
                }
            }
            Event::TaskListMarker(b) => {
                if let Some(top) = stack.last_mut() {
                    top.inlines.push(Inline::Text(Region::from_str(if *b {
                        "[x]"
                    } else {
                        "[ ]"
                    })));
                } else {
                    out.push(Block::Paragraph(vec![Inline::Text(Region::from_str(
                        if *b { "[x]" } else { "[ ]" },
                    ))]));
                }
            }
            Event::FootnoteReference(t) => {
                let s = t.to_string();
                if let Some(top) = stack.last_mut() {
                    top.inlines.push(Inline::FootnoteReference(s));
                } else {
                    out.push(Block::Paragraph(vec![Inline::FootnoteReference(s)]));
                }
            }
            Event::InlineMath(t) => {
                let r = region_from_cow(t);
                if let Some(top) = stack.last_mut() {
                    top.inlines.push(Inline::InlineMath(r));
                } else {
                    out.push(Block::Paragraph(vec![Inline::InlineMath(r)]));
                }
            }
            Event::DisplayMath(t) => {
                let r = region_from_cow(t);
                if let Some(top) = stack.last_mut() {
                    top.inlines.push(Inline::DisplayMath(r));
                } else {
                    out.push(Block::Paragraph(vec![Inline::DisplayMath(r)]));
                }
            }
        }
    }

    out
}
