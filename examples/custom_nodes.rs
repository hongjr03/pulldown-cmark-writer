//! Example showing how to implement custom Inline and Block nodes and use them
//! with the pulldown-cmark-writer AST.
//! Example showing how to implement custom Inline and Block nodes and use them
//! with the pulldown-cmark-writer AST. This demonstrates constructing the
//! AST programmatically (no parsing) and writing it back to Markdown.

use pulldown_cmark::{CowStr, Event, Tag, TagEnd};
use pulldown_cmark_writer::ast::custom::{BlockNode, InlineNode};
use pulldown_cmark_writer::ast::{Block, Inline, writer::blocks_to_markdown};
use std::sync::Arc;

#[derive(Debug, Clone)]
struct MyInline(String);
impl InlineNode for MyInline {
    fn to_events(&self) -> Vec<Event<'static>> {
        vec![
            Event::Start(Tag::Emphasis),
            Event::Text(CowStr::from(self.0.clone())),
            Event::End(TagEnd::Emphasis),
        ]
    }
}

#[derive(Debug, Clone)]
struct MyBlock(String);
impl BlockNode for MyBlock {
    fn to_events(&self) -> Vec<Event<'static>> {
        // render as an HTML block so the writer will include it as-is
        vec![Event::Html(CowStr::from(self.0.clone()))]
    }
}

fn main() {
    // Construct a paragraph containing a custom inline node
    let inl = Inline::Custom(Arc::new(MyInline("hello custom".to_string())));
    let para = Block::Paragraph(vec![inl]);

    // Construct a custom block node (HTML fragment)
    let custom_block = Block::Custom(Arc::new(MyBlock(
        "<div class=\"note\">custom block</div>".to_string(),
    )));

    let blocks = vec![para, custom_block];

    let md = blocks_to_markdown(&blocks);
    println!("Generated Markdown:\n---\n{}\n---", md);
}
