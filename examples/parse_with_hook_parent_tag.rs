//! Example showing how to use a parse hook that matches only when the
//! parent tag is `Tag::HtmlBlock`. This demonstrates precise matching using
//! `ctx.parent_tag` (now a `Tag<'static>`).

use std::sync::Arc;

use pulldown_cmark::{Event, Parser, Tag};
use pulldown_cmark_writer::ast::Block;
use pulldown_cmark_writer::ast::ParseContext;
use pulldown_cmark_writer::ast::custom::BlockNode;
use pulldown_cmark_writer::ast::parse::parse_events_to_blocks_with_hook;
use pulldown_cmark_writer::ast::writer::blocks_to_markdown;

#[derive(Debug)]
struct FigureBlock {
    html: String,
}
impl BlockNode for FigureBlock {
    fn to_events(&self) -> Vec<Event<'static>> {
        vec![Event::Html(self.html.clone().into())]
    }
}

fn main() {
    let md = r#"Intro.

<div class="note">
<figure>
<svg><!-- svg --></svg>
<figcaption>Caption</figcaption>
</figure>
</div>

Tail text.
"#;

    let parser = Parser::new(md);
    let events: Vec<Event> = parser.collect();

    // Hook: only run when we're inside an HtmlBlock (parent_tag == Tag::HtmlBlock).
    let mut hook = |evs: &[Event], _idx: usize, ctx: &ParseContext| -> Option<(usize, Block)> {
        if evs.is_empty() {
            return None;
        }
        // Precise parent tag match
        if let Some(parent) = ctx.parent_tag.as_ref() {
            if !matches!(parent, Tag::HtmlBlock) {
                return None;
            }
        } else {
            return None;
        }

        // We're inside an HtmlBlock; if the first Html event contains <figure>,
        // consume until </figure> across consecutive Html events and return a custom block.
        match &evs[0] {
            Event::Html(h0) => {
                let mut acc = h0.to_string();
                if acc.contains("<figure") {
                    if acc.contains("</figure>") {
                        let fig = FigureBlock { html: acc };
                        return Some((1, Block::Custom(Arc::new(fig))));
                    }
                    let mut end_idx: Option<usize> = None;
                    for (j, e) in evs.iter().enumerate().skip(1) {
                        if let Event::Html(hj) = e {
                            acc.push_str(&hj.to_string());
                            if acc.contains("</figure>") {
                                end_idx = Some(j);
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                    if let Some(m) = end_idx {
                        let fig = FigureBlock { html: acc };
                        return Some((m + 1, Block::Custom(Arc::new(fig))));
                    }
                }
                None
            }
            _ => None,
        }
    };

    let blocks = parse_events_to_blocks_with_hook(&events, Some(&mut hook));
    let out = blocks_to_markdown(&blocks);
    println!("Parsed and rendered (parent_tag match):\n---\n{}\n---", out);
}
