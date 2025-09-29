use pulldown_cmark::{CowStr, Event, Tag};
use pulldown_cmark_writer::ast::custom::{BlockNode, InlineNode};
use pulldown_cmark_writer::ast::{
    Block, Inline, block_to_events, inline_to_events, writer::blocks_to_markdown,
};
use std::sync::Arc;

// A simple custom inline node that renders as emphasized text containing its payload.
#[derive(Debug, Clone)]
struct MyInline(String);
impl InlineNode for MyInline {
    fn to_events(&self) -> Vec<Event<'static>> {
        vec![
            Event::Start(Tag::Emphasis),
            Event::Text(CowStr::from(self.0.clone())),
            Event::End(pulldown_cmark::TagEnd::Emphasis),
        ]
    }
}

// A simple custom block node that renders an HTML block with provided content.
#[derive(Debug, Clone)]
struct MyBlock(String);
impl BlockNode for MyBlock {
    fn to_events(&self) -> Vec<Event<'static>> {
        vec![Event::Html(CowStr::from(self.0.clone()))]
    }
}

#[test]
fn custom_nodes_roundtrip() {
    // Create an Inline::Custom with emphasis around "hello"
    let inl = Inline::Custom(Arc::new(MyInline("hello".to_string())));
    // Convert inline to events and ensure we got Emphasis start/Text/End
    let evs = inline_to_events(&inl);
    assert!(matches!(evs.get(0), Some(Event::Start(Tag::Emphasis))));
    assert!(matches!(evs.get(1), Some(Event::Text(_))));
    assert!(matches!(evs.get(2), Some(Event::End(_))));

    // Create a block that contains our custom inline
    let para = Block::Paragraph(vec![inl.clone()]);
    let md = blocks_to_markdown(&[para]);
    // the writer flattens inline emphasis into markdown '*' markers; since our custom inline emitted Emphasis events,
    // the writer will treat them as inlines when converting paragraph -> region. The exact flatten may produce '*' around text.
    assert!(md.contains("hello"));

    // Test Block::Custom that returns Html event
    let b = Block::Custom(Arc::new(MyBlock("<div>custom</div>".to_string())));
    let events = block_to_events(&b);
    assert!(matches!(events.as_slice(), [Event::Html(_)]));
}
