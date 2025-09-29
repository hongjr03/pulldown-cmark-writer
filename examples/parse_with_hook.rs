//! Example showing how to use a parse hook to map an HTML <figure> block to a custom Block::Custom

use std::sync::Arc;

use pulldown_cmark::{Event, Parser};
use pulldown_cmark_writer::ast::Block;
use pulldown_cmark_writer::ast::custom::BlockNode;
use pulldown_cmark_writer::ast::parse::parse_events_to_blocks_with_hook;
use pulldown_cmark_writer::ast::writer::blocks_to_markdown;
use pulldown_cmark_writer::ast::ParseContext;

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
    let md = r#"Some intro text.

<div class="note">
<figure>
<svg><!-- ... --></svg>
<figcaption>Caption</figcaption>
</figure>
</div>

More text.
"#;

    let parser = Parser::new(md);
    let events: Vec<Event> = parser.collect();

    // (example) raw events omitted for brevity

    // our hook: if we see an Html event that contains <figure> at top-level,
    // consume it as a custom block
    let mut hook = |evs: &[Event], _idx: usize, ctx: &ParseContext| -> Option<(usize, Block)> {
        if evs.is_empty() {
            return None;
        }
        // only run at top-level by default (depth == 0)
        if ctx.depth != 0 {
            return None;
        }
        // If the current event is HTML containing the start of a <figure>,
        // scan forward through subsequent Html events until we find the
        // closing </figure>, then consume that range and return a custom Block.
        match &evs[0] {
            Event::Html(h0) => {
                let mut acc = h0.to_string();
                if acc.contains("<figure") {
                    // if the initial fragment already contains the closing tag, consume it
                    if acc.contains("</figure>") {
                        let fig = FigureBlock { html: acc };
                        return Some((1, Block::Custom(Arc::new(fig))));
                    }
                    // otherwise try to find the matching closing fragment in subsequent Html events
                    let mut end_idx: Option<usize> = None;
                    for (j, e) in evs.iter().enumerate().skip(1) {
                        if let Event::Html(hj) = e {
                            acc.push_str(&hj.to_string());
                            if acc.contains("</figure>") {
                                end_idx = Some(j);
                                break;
                            }
                        } else {
                            // if non-HTML event encountered, stop scanning to avoid
                            // consuming unrelated events
                            break;
                        }
                    }
                    if let Some(m) = end_idx {
                        let fig = FigureBlock { html: acc };
                        return Some((m + 1, Block::Custom(Arc::new(fig))));
                    }
                    // no closing tag found yet: don't consume; let normal parser handle
                    return None;
                }
                None
            }
            _ => None,
        }
    };

    let blocks = parse_events_to_blocks_with_hook(&events, Some(&mut hook));
    let out = blocks_to_markdown(&blocks);
    println!("Parsed and rendered:\n---\n{}\n---", out);
}
