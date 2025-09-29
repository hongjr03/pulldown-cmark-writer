//! Example demonstrating custom block and inline nodes: a <figure> with inline SVG
//! and an inline badge image implemented as custom nodes.

use std::sync::Arc;

use pulldown_cmark::Event;

use pulldown_cmark_writer::ast::Block;
use pulldown_cmark_writer::ast::Inline;
use pulldown_cmark_writer::ast::custom::{BlockNode, InlineNode};
use pulldown_cmark_writer::ast::writer::blocks_to_markdown;

#[derive(Debug)]
struct BadgeInline {
    url: String,
    alt: String,
}

impl InlineNode for BadgeInline {
    fn to_events(&self) -> Vec<Event<'static>> {
        // Render as an inline image using raw markdown events: ![alt](url)
        vec![
            Event::Start(pulldown_cmark::Tag::Image {
                link_type: pulldown_cmark::LinkType::Inline,
                dest_url: self.url.clone().into(),
                title: "".into(),
                id: "".into(),
            }),
            Event::Text(self.alt.clone().into()),
            Event::End(pulldown_cmark::TagEnd::Image),
        ]
    }
}

#[derive(Debug)]
struct FigureBlock {
    svg: String,
    caption: String,
}

impl BlockNode for FigureBlock {
    fn to_events(&self) -> Vec<Event<'static>> {
        // Emit the whole figure as an HTML block so it remains verbatim.
        // Use Event::Html which writer treats as HtmlBlock content.
        vec![Event::Html(
            format!(
                "<figure>\n{}\n<figcaption>{}</figcaption>\n</figure>",
                self.svg, self.caption
            )
            .into(),
        )]
    }
}

fn main() {
    // build a paragraph: "This is a badge " + BadgeInline + " in text"
    let para = vec![
        Inline::Text(pulldown_cmark_writer::text::Region::from_str("This is a ")),
        Inline::Custom(Arc::new(BadgeInline {
            url: "https://example.com/badge.svg".into(),
            alt: "badge".into(),
        })),
        Inline::Text(pulldown_cmark_writer::text::Region::from_str(" in text.")),
    ];

    // simple SVG
    let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="120" height="30">
        <rect width="120" height="30" fill="#555"/>
        <text x="10" y="20" fill="#fff">Example</text>
    </svg>"##
        .to_string();

    let fig = Block::Custom(Arc::new(FigureBlock {
        svg,
        caption: "Figure 1: Example SVG".into(),
    }));

    let blocks = vec![Block::Paragraph(para), fig];

    let md = blocks_to_markdown(&blocks);
    println!("Generated Markdown:\n---\n{}\n---", md);
}
