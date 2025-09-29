use pulldown_cmark::{Parser, Options, Event};
use pulldown_cmark_writer::ast::{parse_events_to_blocks, block_to_events, blocks_to_markdown};
use std::fs;

fn main() {
    let path = "src/fixtures/specs/math/example7.md";
    let s = fs::read_to_string(path).expect("read fixture");
    println!("--- ORIGINAL FILE ---\n{}\n--- END ---", s);
    let parser = Parser::new_ext(&s, Options::empty());
    let events: Vec<Event> = parser.collect();
    println!("ORIGINAL PARSER EVENTS:");
    for ev in &events {
        println!("{:?}", ev);
    }

    let events_static: Vec<Event<'static>> = events.into_iter().map(|e| e.into_static()).collect();
    let ast = parse_events_to_blocks(&events_static);
    let mut out_events: Vec<Event<'static>> = Vec::new();
    for b in &ast {
        out_events.extend(block_to_events(b));
    }
    println!("OUT EVENTS from AST:");
    for ev in &out_events {
        println!("{:?}", ev);
    }

    let md = blocks_to_markdown(&ast);
    println!("--- GENERATED MARKDOWN ---\n{}\n--- END ---", md);

    let p2 = Parser::new_ext(&md, Options::empty());
    let events_md: Vec<Event> = p2.collect();
    println!("PARSER EVENTS FOR GENERATED MD:");
    for ev in &events_md {
        println!("{:?}", ev);
    }
}
