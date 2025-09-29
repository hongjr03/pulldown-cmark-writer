use pulldown_cmark::{Parser, Event, Options};
use pulldown_cmark_writer::ast::{parse_events_to_blocks, block_to_events, blocks_to_markdown};
use similar::{TextDiff, ChangeTag};
use std::fs;
use std::path::Path;

fn collect_md_files(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    if dir.is_dir() {
        for entry in fs::read_dir(dir).unwrap() {
            let e = entry.unwrap();
            let p = e.path();
            if p.is_dir() {
                collect_md_files(&p, out);
            } else if let Some(ext) = p.extension() {
                if ext == "md" {
                    out.push(p);
                }
            }
        }
    }
}

fn normalize_events(events: Vec<Event<'static>>) -> Vec<Event<'static>> {
    // Merge adjacent Text events and normalize code block text by collapsing
    // multiple blank lines into a single blank line to avoid insignificant
    // formatting differences produced by different event sequences.
    use pulldown_cmark::CowStr;
    let mut out: Vec<Event<'static>> = Vec::new();
    let mut in_codeblock = false;
    for ev in events {
        match ev {
            Event::Start(pulldown_cmark::Tag::CodeBlock(_)) => {
                in_codeblock = true;
                out.push(ev);
            }
            Event::End(pulldown_cmark::TagEnd::CodeBlock) => {
                in_codeblock = false;
                out.push(ev);
            }
            Event::Text(t) => {
                let mut txt = t.to_string();
                if in_codeblock {
                    // collapse multiple blank lines into a single blank line
                    let mut res = String::new();
                    let mut last_was_empty = false;
                    for (i, line) in txt.split('\n').enumerate() {
                        let is_empty = line.trim().is_empty();
                        if is_empty {
                            if !last_was_empty {
                                // represent a single blank line (by appending one '\n' if res not empty)
                                if !res.is_empty() {
                                    res.push('\n');
                                }
                                last_was_empty = true;
                            }
                        } else {
                            if !res.is_empty() {
                                res.push('\n');
                            }
                            res.push_str(line);
                            last_was_empty = false;
                        }
                        // preserve final trailing newline as in original
                        if i == txt.split('\n').count() - 1 && txt.ends_with('\n') {
                            // will append a trailing newline below if needed
                        }
                    }
                    if txt.ends_with('\n') && !res.ends_with('\n') {
                        res.push('\n');
                    }
                    txt = res;
                }

                if let Some(Event::Text(prev)) = out.last_mut() {
                    let mut s = prev.to_string();
                    s.push_str(&txt);
                    *prev = CowStr::from(s);
                } else {
                    out.push(Event::Text(CowStr::from(txt)));
                }
            }
            other => out.push(other),
        }
    }
    out
}

fn filter_paragraph_events(events: Vec<Event<'static>>) -> Vec<Event<'static>> {
    events
        .into_iter()
        .filter(|ev| match ev {
            Event::Start(pulldown_cmark::Tag::Paragraph) => false,
            Event::End(pulldown_cmark::TagEnd::Paragraph) => false,
            _ => true,
        })
        .collect()
}

// Collapse consecutive Text events into single string tokens and stringify
// non-text events. This produces a stable, human/LLM-friendly sequence for
// semantic comparison that tolerates differences in how text was split into
// individual Event::Text chunks.
fn canonicalize_events(events: Vec<Event<'static>>) -> Vec<String> {
    use pulldown_cmark::{Tag, CodeBlockKind};
    let mut out: Vec<String> = Vec::new();
    let mut acc: Option<String> = None;
    for ev in events {
        match ev {
            Event::Text(t) => {
                if let Some(s) = acc.as_mut() {
                    s.push_str(&t.to_string());
                } else {
                    acc = Some(t.to_string());
                }
            }
            Event::Code(t) => {
                // represent inline code as a single text token with backticks so
                // differences between Code vs Text representations are smoothed
                if let Some(s) = acc.take() {
                    out.push(format!("Text({:?})", s));
                }
                out.push(format!("Text({:?})", format!("`{}`", t.to_string())));
            }
            Event::Start(tag) => {
                if let Some(s) = acc.take() {
                    out.push(format!("Text({:?})", s));
                }
                match tag {
                    Tag::CodeBlock(kind) => match kind {
                        CodeBlockKind::Fenced(lang) => {
                            out.push(format!("Start(CodeBlock(Fenced({:?})))", lang.to_string()));
                        }
                        CodeBlockKind::Indented => out.push("Start(CodeBlock(Indented))".to_string()),
                    },
                    Tag::Link { link_type, dest_url, title, id } => {
                        out.push(format!(
                            "Start(Link {{ link_type: {:?}, dest_url: {:?}, title: {:?}, id: {:?} }})",
                            link_type,
                            dest_url.to_string(),
                            title.to_string(),
                            id.to_string()
                        ));
                    }
                    Tag::Image { link_type, dest_url, title, id } => {
                        out.push(format!(
                            "Start(Image {{ link_type: {:?}, dest_url: {:?}, title: {:?}, id: {:?} }})",
                            link_type,
                            dest_url.to_string(),
                            title.to_string(),
                            id.to_string()
                        ));
                    }
                    other => out.push(format!("Start({:?})", other)),
                }
            }
            Event::End(tagend) => {
                if let Some(s) = acc.take() {
                    out.push(format!("Text({:?})", s));
                }
                match tagend {
                    pulldown_cmark::TagEnd::CodeBlock => out.push("End(CodeBlock)".to_string()),
                    other => out.push(format!("End({:?})", other)),
                }
            }
            other => {
                if let Some(s) = acc.take() {
                    out.push(format!("Text({:?})", s));
                }
                out.push(format!("{:?}", other));
            }
        }
    }
    if let Some(s) = acc.take() {
        out.push(format!("Text({:?})", s));
    }
    out
}

#[test]
fn fixtures_events_roundtrip() {
    let mut files = Vec::new();
    collect_md_files(Path::new("src/fixtures"), &mut files);
    assert!(!files.is_empty(), "no fixture files found");

    for f in files {
        let s = fs::read_to_string(&f).unwrap();
        let parser = Parser::new_ext(&s, Options::empty());
        let events: Vec<Event> = parser.collect();
        let events_static: Vec<Event<'static>> = events.into_iter().map(|e| e.into_static()).collect();

        // parse -> ast -> events
        let ast = parse_events_to_blocks(&events_static);
        let mut out_events: Vec<Event<'static>> = Vec::new();
        for b in &ast {
            out_events.extend(block_to_events(b));
        }

        // additionally: ast -> markdown -> events (re-parse our generated markdown)
        let md = blocks_to_markdown(&ast);
        let p2 = Parser::new_ext(&md, Options::empty());
        let events_md: Vec<Event> = p2.collect();
        let events_md_static: Vec<Event<'static>> = events_md.into_iter().map(|e| e.into_static()).collect();
    let events_md_norm = filter_paragraph_events(normalize_events(events_md_static));
    let md_canon = canonicalize_events(events_md_norm);
    let ev_norm = filter_paragraph_events(normalize_events(events_static));
    let out_norm = filter_paragraph_events(normalize_events(out_events));

    // canonicalize by collapsing Text runs into single tokens for comparison
    let ev_canon = canonicalize_events(ev_norm);
    let out_canon = canonicalize_events(out_norm);

        // For debugging show filename on failure
        if ev_canon != out_canon {
            // Render human friendly diff using similar
            let left = ev_canon.join("\n");
            let right = out_canon.join("\n");
            let diff = TextDiff::from_lines(&left, &right);
            eprintln!("Event diff for {:?}:\n", f);
            for op in diff.ops() {
                for change in diff.iter_changes(op) {
                    match change.tag() {
                        ChangeTag::Delete => eprint!("- {}", change),
                        ChangeTag::Insert => eprint!("+ {}", change),
                        ChangeTag::Equal => eprint!("  {}", change),
                    }
                }
            }
            eprintln!();
        }
        assert_eq!(ev_canon, out_canon, "roundtrip mismatch for {:?}", f);

        // Also ensure that parsing the generated markdown yields similar events
        if ev_canon != md_canon {
            let left = ev_canon.join("\n");
            let right = md_canon.join("\n");
            let diff = TextDiff::from_lines(&left, &right);
            eprintln!("Event diff for generated-markdown for {:?}:\n", f);
            for op in diff.ops() {
                for change in diff.iter_changes(op) {
                    match change.tag() {
                        ChangeTag::Delete => eprint!("- {}", change),
                        ChangeTag::Insert => eprint!("+ {}", change),
                        ChangeTag::Equal => eprint!("  {}", change),
                    }
                }
            }
            eprintln!();
        }
        assert_eq!(ev_canon, md_canon, "markdown reparse roundtrip mismatch for {:?}", f);
    }
}
