//! Rich example demonstrating direct AST construction with custom nodes in pulldown-cmark-writer.
//! This example shows:
//! - Directly constructing AST blocks and inlines
//! - Using custom block and inline nodes
//! - Writing the AST back to Markdown and HTML

use pulldown_cmark::{
    Alignment, CodeBlockKind, CowStr, Event, HeadingLevel, LinkType, Tag, TagEnd, html,
};
use pulldown_cmark_writer::ast::custom::{BlockNode, InlineNode};
use pulldown_cmark_writer::ast::{Block, Inline, block_to_events, writer::blocks_to_markdown};
use pulldown_cmark_writer::text::Region;
use std::sync::Arc;

// Custom inline node: renders as bold text
#[derive(Debug, Clone)]
struct BoldInline(String);
impl InlineNode for BoldInline {
    fn to_events(&self) -> Vec<Event<'static>> {
        vec![
            Event::Start(Tag::Strong),
            Event::Text(CowStr::from(self.0.clone())),
            Event::End(TagEnd::Strong),
        ]
    }
}

// Custom block node: renders as a warning blockquote
#[derive(Debug, Clone)]
struct WarningBlock {
    title: String,
    content: Vec<Block>,
}
impl BlockNode for WarningBlock {
    fn to_events(&self) -> Vec<Event<'static>> {
        let mut events = vec![
            Event::Start(Tag::BlockQuote(None)),
            Event::Start(Tag::Paragraph),
            Event::Text(CowStr::from(format!("⚠️ **{}**", self.title))),
            Event::End(TagEnd::Paragraph),
        ];

        // Add content blocks
        for block in &self.content {
            events.extend(block_to_events(block));
        }

        events.push(Event::End(TagEnd::BlockQuote(None)));
        events
    }
}

fn main() {
    // Directly construct AST blocks

    // Heading
    let heading = Block::Heading {
        level: HeadingLevel::H1,
        id: None,
        classes: vec![],
        attrs: vec![],
        children: vec![Inline::Text(Region::from_str("Hello World"))],
    };

    // Paragraph with custom inline
    let paragraph = Block::Paragraph(vec![
        Inline::Text(Region::from_str("This is a paragraph with some ")),
        Inline::Strong(vec![Inline::Text(Region::from_str("bold"))]),
        Inline::Text(Region::from_str(" text and a ")),
        Inline::Link {
            link_type: LinkType::Inline,
            dest: "https://example.com".to_string(),
            title: "".to_string(),
            id: "".to_string(),
            children: vec![Inline::Text(Region::from_str("link"))],
        },
        Inline::Text(Region::from_str(".")),
        Inline::Text(Region::from_str(" And this is ")),
        Inline::Custom(Arc::new(BoldInline("custom bold text".to_string()))),
        Inline::Text(Region::from_str(".")),
    ]);

    // Custom warning block
    let warning_content = vec![
        Block::Paragraph(vec![Inline::Text(Region::from_str(
            "This is a warning about something important.",
        ))]),
        Block::List {
            start: None,
            items: vec![
                vec![Block::Paragraph(vec![Inline::Text(Region::from_str(
                    "Point 1",
                ))])],
                vec![Block::Paragraph(vec![Inline::Text(Region::from_str(
                    "Point 2",
                ))])],
            ],
        },
    ];
    let warning_block = Block::Custom(Arc::new(WarningBlock {
        title: "Important Warning".to_string(),
        content: warning_content,
    }));

    // List
    let list = Block::List {
        start: None,
        items: vec![
            vec![Block::Paragraph(vec![Inline::Text(Region::from_str(
                "Item 1",
            ))])],
            vec![
                Block::Paragraph(vec![Inline::Text(Region::from_str("Item 2"))]),
                Block::List {
                    start: None,
                    items: vec![vec![Block::Paragraph(vec![Inline::Text(
                        Region::from_str("Nested item"),
                    )])]],
                },
            ],
        ],
    };

    // Code block
    let code_block = Block::CodeBlock {
        kind: CodeBlockKind::Fenced("rust".into()),
        content: Region::from_str("fn main() {\n    println!(\"Hello, world!\");\n}"),
    };

    // Table
    let table = Block::Table(
        vec![Alignment::None, Alignment::None],
        vec![
            vec![
                vec![Inline::Text(Region::from_str("Header 1"))],
                vec![Inline::Text(Region::from_str("Header 2"))],
            ],
            vec![
                vec![Inline::Text(Region::from_str("Cell 1"))],
                vec![Inline::Text(Region::from_str("Cell 2"))],
            ],
        ],
    );

    let blocks = vec![heading, paragraph, warning_block, list, code_block, table];

    // Write back to Markdown
    let output_markdown = blocks_to_markdown(&blocks);
    println!("Constructed Markdown:\n---\n{}\n---", output_markdown);

    // Write to HTML
    let mut html_output = String::new();
    let mut events = Vec::new();
    for block in &blocks {
        events.extend(block_to_events(block));
    }
    html::push_html(&mut html_output, events.into_iter());
    println!("Constructed HTML:\n---\n{}\n---", html_output);
}
