use pulldown_cmark::Parser;

fn print_events(s: &str) {
    println!("--- input ---\n{}\n--- events ---", s);
    let parser = Parser::new_ext(s, pulldown_cmark::Options::empty());
    for ev in parser {
        println!("{:?}", ev);
    }
    println!("--- end ---\n");
}

fn main() {
    let tests = vec![
        r#"| A | B \\| |
|---|---|
| x | y |"#,
        r#"| A | B \\\\| |
|---|---|
| x | y |"#,
        r#"| A | B \\\\\\| |
|---|---|
| x | y |"#,
        r#"| Wait, what? |         \|"#,
        r#"| Wait, what? |        \|"#,
    ];
    for t in tests {
        print_events(t);
    }
}
