use pulldown_cmark::{Parser, Event};
use pulldown_cmark_writer::ast;
use pulldown_cmark_writer::ast::writer::blocks_to_markdown;
use pulldown_cmark_writer::ast::custom::BlockParser;
use std::sync::Arc;

// A tiny parser that recognizes a <figure>...</figure> HTML block (emitted as Event::Html)
// and converts it into a custom Block::Custom node that serializes back to the same HTML.
#[derive(Debug)]
struct FigureParser;

impl BlockParser for FigureParser {
    fn try_parse(&self, events: &[Event], idx: usize, _ctx: &crate::ast::ParseContext) -> Option<(usize, crate::ast::Block)> {
        // Look for a sequence: Html("<figure>") ... Html("</figure>")
        if idx >= events.len() {
            return None;
        }
        match &events[idx] {
            Event::Html(s) if s.trim_start().starts_with("<figure") => {
                // scan forward for a closing </figure> html event
                for j in idx+1..events.len() {
                    if let Event::Html(end) = &events[j] {
                        if end.trim_start().starts_with("</figure") {
                            // Collect the raw HTML lines between idx..=j
                            let mut content = String::new();
                            for k in idx..=j {
                                if let Event::Html(part) = &events[k] {
                                    content.push_str(part);
                                }
                            }
                            // Create a block that roundtrips via Custom BlockNode
                            #[derive(Debug)]
                            struct RawHtmlBlock(String);
                            impl pulldown_cmark_writer::ast::custom::BlockNode for RawHtmlBlock {
                                fn to_events(&self) -> Vec<Event<'static>> {
                                    vec![Event::Html(self.0.clone().into())]
                                }
                            }
                            let rb = RawHtmlBlock(content);
                            let blk = crate::ast::Block::Custom(Arc::new(rb));
                            return Some((j - idx + 1, blk));
                        }
                    }
                }
                None
            }
            _ => None,
        }
    }
}

fn main() {
    let src = r#"
<div>
  <figure>
    <img src=\"/img.png\" />
    <figcaption>Caption</figcaption>
  </figure>
</div>
"#;
    let parser = Parser::new(src);
    let events: Vec<Event> = parser.collect();

    // instantiate our parser and run the parse-with-parsers wrapper
    let figure = FigureParser;
    let parsers: [&dyn BlockParser; 1] = [&figure];
    let blocks = ast::parse_events_to_blocks_with_parsers(&events, &parsers);

    println!("Parsed blocks (debug):\n{:?}\n", blocks);

    let md = blocks_to_markdown(&blocks);
    println!("Rendered markdown:\n{}", md);
}
