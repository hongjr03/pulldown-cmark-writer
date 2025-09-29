use crate::ast::inline::{Inline, inline_to_events};
use crate::ast::custom::BlockNode;
use std::sync::Arc;
use crate::text::Region;
use pulldown_cmark::{Alignment, CodeBlockKind, CowStr, Event, HeadingLevel, Tag, TagEnd};

/// Block level AST nodes.
#[derive(Clone, Debug)]
pub enum Block {
    Paragraph(Vec<Inline>),
    Heading {
        level: HeadingLevel,
        id: Option<String>,
        classes: Vec<String>,
        attrs: Vec<(String, Option<String>)>,
        children: Vec<Inline>,
    },
    BlockQuote(Vec<Block>),
    CodeBlock {
        kind: CodeBlockKind<'static>,
        content: Region,
    },
    HtmlBlock(Region),
    List {
        start: Option<u64>,
        items: Vec<Vec<Block>>,
    },
    Item(Vec<Block>),
    Rule,
    FootnoteDefinition(String, Vec<Block>),
    Table(Vec<Alignment>),
    TableRow(Vec<Vec<crate::ast::inline::Inline>>),
    TableFull(Vec<Alignment>, Vec<Vec<Vec<crate::ast::inline::Inline>>>),
    /// A user-provided custom block node.
    Custom(Arc<dyn BlockNode + 'static>),
}

/// Convert a `Block` into pulldown-cmark events (owned, 'static).
pub fn block_to_events(b: &Block) -> Vec<Event<'static>> {
    match b {
        Block::Paragraph(children) => {
            let mut out = vec![Event::Start(Tag::Paragraph)];
            for c in children {
                out.extend(inline_to_events(c));
            }
            out.push(Event::End(TagEnd::Paragraph));
            out
        }
        Block::Heading {
            level,
            id,
            classes: _,
            attrs: _,
            children,
        } => {
            let idcow = id.as_ref().map(|s| CowStr::from(s.clone()));
            let mut out = vec![Event::Start(Tag::Heading {
                level: *level,
                id: idcow,
                classes: vec![],
                attrs: vec![],
            })];
            for c in children {
                out.extend(inline_to_events(c));
            }
            out.push(Event::End(TagEnd::Heading(*level)));
            out
        }
        Block::BlockQuote(children) => {
            let mut out = vec![Event::Start(Tag::BlockQuote(None))];
            for ch in children {
                out.extend(block_to_events(ch));
            }
            out.push(Event::End(TagEnd::BlockQuote(None)));
            out
        }
        Block::CodeBlock { kind, content } => {
            let mut out = vec![Event::Start(Tag::CodeBlock(kind.clone()))];
            // each line as Html/Text event is fine; we emit a single Text event
            out.push(Event::Text(CowStr::from(content.apply())));
            out.push(Event::End(TagEnd::CodeBlock));
            out
        }
        Block::HtmlBlock(r) => vec![Event::Html(CowStr::from(r.apply()))],
        Block::List { start, items } => {
            let mut out = vec![Event::Start(Tag::List(*start))];
            for item in items {
                out.push(Event::Start(Tag::Item));
                for ch in item {
                    out.extend(block_to_events(ch));
                }
                out.push(Event::End(TagEnd::Item));
            }
            out.push(Event::End(TagEnd::List(start.is_some())));
            out
        }
        Block::Item(children) => {
            let mut out = vec![Event::Start(Tag::Item)];
            for ch in children {
                out.extend(block_to_events(ch));
            }
            out.push(Event::End(TagEnd::Item));
            out
        }
        Block::Rule => vec![Event::Rule],
        Block::FootnoteDefinition(label, children) => {
            let mut out = vec![Event::Start(Tag::FootnoteDefinition(CowStr::from(
                label.clone(),
            )))];
            for ch in children {
                out.extend(block_to_events(ch));
            }
            out.push(Event::End(TagEnd::FootnoteDefinition));
            out
        }
        Block::Table(_aligns) => vec![],
        Block::TableRow(_) => vec![],
        Block::TableFull(_, _) => vec![],
        Block::Custom(c) => c.to_events(),
    }
}
