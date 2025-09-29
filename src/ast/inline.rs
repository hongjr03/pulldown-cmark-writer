use crate::ast::custom::InlineNode;
use crate::text::Region;
use pulldown_cmark::{CowStr, Event, Tag, TagEnd};
use std::sync::Arc;

/// Inline level AST nodes. They own their text via `Region` which composes
/// `Line`/`Fragment` from `src/text`.
#[derive(Clone, Debug)]
pub enum Inline {
    Text(Region),
    Code(Region),
    InlineHtml(Region),
    Html(Region),
    SoftBreak,
    HardBreak,
    Emphasis(Vec<Inline>),
    Strong(Vec<Inline>),
    Strikethrough(Vec<Inline>),
    Subscript(Vec<Inline>),
    Superscript(Vec<Inline>),
    Link {
        link_type: pulldown_cmark::LinkType,
        dest: String,
        title: String,
        id: String,
        children: Vec<Inline>,
    },
    Image {
        link_type: pulldown_cmark::LinkType,
        dest: String,
        title: String,
        id: String,
        children: Vec<Inline>,
    },
    FootnoteReference(String),
    InlineMath(Region),
    DisplayMath(Region),
    /// A user-provided custom inline node. Boxed trait object so the AST
    /// can carry arbitrary user types that implement `InlineNode`.
    Custom(Arc<dyn InlineNode + 'static>),
}

/// Convert `Inline` to a sequence of pulldown-cmark Events (owned, 'static).
pub fn inline_to_events(inl: &Inline) -> Vec<Event<'static>> {
    match inl {
        Inline::Text(r) => {
            let s = r.apply();
            // preserve soft breaks as explicit events by splitting on '\n'
            let mut out: Vec<Event<'static>> = Vec::new();
            let parts: Vec<&str> = s.split('\n').collect();
            for (i, part) in parts.iter().enumerate() {
                out.push(Event::Text(CowStr::from((*part).to_string())));
                if i + 1 < parts.len() {
                    out.push(Event::SoftBreak);
                }
            }
            out
        }
        Inline::Code(r) => vec![Event::Code(CowStr::from(r.apply()))],
        Inline::InlineHtml(r) => vec![Event::InlineHtml(CowStr::from(r.apply()))],
        Inline::Html(r) => vec![Event::Html(CowStr::from(r.apply()))],
        Inline::SoftBreak => vec![Event::SoftBreak],
        Inline::HardBreak => vec![Event::HardBreak],
        Inline::Emphasis(children) => {
            let mut out = vec![Event::Start(Tag::Emphasis)];
            for c in children {
                out.extend(inline_to_events(c));
            }
            out.push(Event::End(TagEnd::Emphasis));
            out
        }
        Inline::Strong(children) => {
            let mut out = vec![Event::Start(Tag::Strong)];
            for c in children {
                out.extend(inline_to_events(c));
            }
            out.push(Event::End(TagEnd::Strong));
            out
        }
        Inline::Strikethrough(children) => {
            let mut out = vec![Event::Start(Tag::Strikethrough)];
            for c in children {
                out.extend(inline_to_events(c));
            }
            out.push(Event::End(TagEnd::Strikethrough));
            out
        }
        Inline::Subscript(children) => {
            let mut out = vec![Event::Start(Tag::Subscript)];
            for c in children {
                out.extend(inline_to_events(c));
            }
            out.push(Event::End(TagEnd::Subscript));
            out
        }
        Inline::Superscript(children) => {
            let mut out = vec![Event::Start(Tag::Superscript)];
            for c in children {
                out.extend(inline_to_events(c));
            }
            out.push(Event::End(TagEnd::Superscript));
            out
        }
        Inline::Link {
            link_type,
            dest,
            title,
            id,
            children,
        } => {
            let mut out = vec![Event::Start(Tag::Link {
                link_type: *link_type,
                dest_url: CowStr::from(dest.clone()),
                title: CowStr::from(title.clone()),
                id: CowStr::from(id.clone()),
            })];
            for c in children {
                out.extend(inline_to_events(c));
            }
            out.push(Event::End(TagEnd::Link));
            out
        }
        Inline::Image {
            link_type,
            dest,
            title,
            id,
            children,
        } => {
            let mut out = vec![Event::Start(Tag::Image {
                link_type: *link_type,
                dest_url: CowStr::from(dest.clone()),
                title: CowStr::from(title.clone()),
                id: CowStr::from(id.clone()),
            })];
            for c in children {
                out.extend(inline_to_events(c));
            }
            out.push(Event::End(TagEnd::Image));
            out
        }
        Inline::FootnoteReference(s) => vec![Event::FootnoteReference(CowStr::from(s.clone()))],
        Inline::InlineMath(r) => vec![Event::InlineMath(CowStr::from(r.apply()))],
        Inline::DisplayMath(r) => vec![Event::DisplayMath(CowStr::from(r.apply()))],
        Inline::Custom(c) => c.to_events(),
    }
}
