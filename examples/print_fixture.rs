use pulldown_cmark::{Parser, Options};
use pulldown_cmark_writer::ast::{parse_events_to_blocks, blocks_to_markdown};
use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: print_fixture <path>");
        return;
    }
    let p = &args[1];
    let s = fs::read_to_string(p).expect("read");
    let parser = Parser::new_ext(&s, Options::empty());
    let events: Vec<_> = parser.collect();
    eprintln!("--- raw events ---");
    for (i, ev) in events.iter().enumerate() {
        eprintln!("{:03}: {:?}", i, ev);
    }
    eprintln!("--- end events ---");
    let events_static: Vec<pulldown_cmark::Event<'static>> = events.into_iter().map(|e| e.into_static()).collect();
    let ast = parse_events_to_blocks(&events_static);
    let md = blocks_to_markdown(&ast);
    println!("--- generated markdown ---");
    for (i, line) in md.split('\n').enumerate() {
        println!("{:03}: {:?}", i + 1, line);
    }
    println!("--- end ---");

    // Debug: render first TableFull block found (un-prefixed)
    for b in &ast {
        fn walk(b: &pulldown_cmark_writer::ast::Block, depth: usize) {
            let pad = " ".repeat(depth * 2);
            match b {
                pulldown_cmark_writer::ast::Block::BlockQuote(children) => {
                    eprintln!("{}BlockQuote ({} children)", pad, children.len());
                    for ch in children {
                        walk(ch, depth + 1);
                    }
                }
                pulldown_cmark_writer::ast::Block::TableFull(aligns, rows) => {
                    eprintln!("{}TableFull cols={} rows={}", pad, aligns.len(), rows.len());
                    for (ri, row) in rows.iter().enumerate() {
                        eprint!("{} row {}:", pad, ri);
                        for cell in row.iter() {
                            let mut s = String::new();
                            for inl in cell {
                                match inl {
                                    pulldown_cmark_writer::ast::Inline::Text(t) => s.push_str(&t.apply()),
                                    _ => s.push_str("[inl]"),
                                }
                            }
                            eprint!(" | {}", s.replace('\n', "\\n"));
                        }
                        eprintln!("");
                    }
                }
                pulldown_cmark_writer::ast::Block::Paragraph(inls) => {
                    eprint!("{}Paragraph: ", pad);
                    for inl in inls {
                        match inl {
                            pulldown_cmark_writer::ast::Inline::Text(t) => eprint!("{:?}", t.apply()),
                            _ => eprint!("[inl]"),
                        }
                    }
                    eprintln!("");
                }
                other => {
                    eprintln!("{}Other: {:?}", pad, other);
                }
            }
        }
        walk(b, 0);
    }
}
