use pulldown_cmark_writer::{Line, Region};
use unicode_width::UnicodeWidthStr;

#[derive(Clone, Copy)]
enum Align {
    Left,
    Center,
    Right,
}

fn pad_to_width(s: &str, width: usize, align: Align) -> String {
    let w = UnicodeWidthStr::width(s);
    if width <= w {
        return s.to_string();
    }
    let pad = width - w;
    match align {
        Align::Left => {
            let mut out = String::from(s);
            out.push_str(&" ".repeat(pad));
            out
        }
        Align::Right => {
            let mut out = String::new();
            out.push_str(&" ".repeat(pad));
            out.push_str(s);
            out
        }
        Align::Center => {
            let left = pad / 2;
            let right = pad - left;
            let mut out = String::new();
            out.push_str(&" ".repeat(left));
            out.push_str(s);
            out.push_str(&" ".repeat(right));
            out
        }
    }
}

fn print_basic_table() {
    println!("# Basic table with unicode-width alignment");
    let header = vec!["Name", "Age", "City"];
    let rows = vec![
        vec!["Alice", "30", "London"],
        vec!["Bob", "25", "Berlin"],
        vec!["陈小龙", "40", "上海"],
    ];

    // Determine widths
    let mut widths = vec![0; header.len()];
    for (i, h) in header.iter().enumerate() {
        widths[i] = widths[i].max(UnicodeWidthStr::width(*h));
    }
    for r in &rows {
        for (i, c) in r.iter().enumerate() {
            widths[i] = widths[i].max(UnicodeWidthStr::width(*c));
        }
    }

    // Build region
    let mut reg = Region::new();

    // Header
    let mut hline = Line::new();
    hline.push("|");
    for (i, h) in header.iter().enumerate() {
        hline.push(pad_to_width(h, widths[i], Align::Center));
        hline.push(" |");
    }
    reg.push_back_line(hline);

    // Separator with alignment markers (center header)
    let mut sep = Line::new();
    sep.push("|");
    for w in &widths {
        sep.push(" ");
        sep.push("-");
        for _ in 0..(*w - 1) {
            sep.push("-");
        }
        sep.push(" |");
    }
    reg.push_back_line(sep);

    // Body
    for r in rows {
        let mut l = Line::new();
        l.push("|");
        for (i, c) in r.iter().enumerate() {
            l.push(pad_to_width(c, widths[i], Align::Left));
            l.push(" |");
        }
        reg.push_back_line(l);
    }

    println!("\n{}\n", reg.apply());
}

fn print_aligned_table() {
    println!("# Table with header/column alignment (L/C/R)");
    let header = vec!["Item", "Price", "Remarks"];
    let rows = vec![
        vec!["Apple", "¥3", "fresh"],
        vec!["Banana", "€1.2", "ripe"],
        vec!["长柄蘑菇", "¥12", "limited stock"],
    ];
    let aligns = vec![Align::Left, Align::Right, Align::Center];

    let mut widths = vec![0; header.len()];
    for (i, h) in header.iter().enumerate() {
        widths[i] = widths[i].max(UnicodeWidthStr::width(*h));
    }
    for r in &rows {
        for (i, c) in r.iter().enumerate() {
            widths[i] = widths[i].max(UnicodeWidthStr::width(*c));
        }
    }

    let mut reg = Region::new();
    // header
    let mut hline = Line::new();
    hline.push("|");
    for (i, h) in header.iter().enumerate() {
        hline.push(pad_to_width(h, widths[i], aligns[i]));
        hline.push(" |");
    }
    reg.push_back_line(hline);

    // separator w/ alignment hints: e.g. :--, :-:, --:
    let mut sep = Line::new();
    sep.push("|");
    for (i, w) in widths.iter().enumerate() {
        let marker = match aligns[i] {
            Align::Left => format!(":{}", "-".repeat(w.saturating_sub(1))),
            Align::Right => format!("{}:", "-".repeat(w.saturating_sub(1))),
            Align::Center => format!(":{}:", "-".repeat(w.saturating_sub(2))),
        };
        sep.push(marker);
        sep.push(" |");
    }
    reg.push_back_line(sep);

    for r in rows {
        let mut l = Line::new();
        l.push("|");
        for (i, c) in r.iter().enumerate() {
            l.push(pad_to_width(c, widths[i], aligns[i]));
            l.push(" |");
        }
        reg.push_back_line(l);
    }

    println!("\n{}\n", reg.apply());
}

fn print_multiline_cells_and_nested_list() {
    println!("# Multi-line cell and nested list example");

    // Multi-line cell: a cell can contain newlines; we split and render row with max lines
    let header = vec!["Task", "Notes"];
    let rows = vec![
        vec!["Build", "compile\nrun tests"],
        vec!["Deploy", "staging\nproduction"],
        vec!["国际化", "翻译\n校对"],
    ];

    let mut widths = vec![0; header.len()];
    for (i, h) in header.iter().enumerate() {
        widths[i] = widths[i].max(UnicodeWidthStr::width(*h));
    }
    for r in &rows {
        for (i, c) in r.iter().enumerate() {
            for line in c.split('\n') {
                widths[i] = widths[i].max(UnicodeWidthStr::width(line));
            }
        }
    }

    // Header
    let mut reg = Region::new();
    let mut hline = Line::new();
    hline.push("|");
    for (i, h) in header.iter().enumerate() {
        hline.push(pad_to_width(h, widths[i], Align::Center));
        hline.push(" |");
    }
    reg.push_back_line(hline);

    // sep
    let mut sep = Line::new();
    sep.push("|");
    for w in &widths {
        sep.push("-");
        for _ in 1..*w {
            sep.push("-");
        }
        sep.push(" |");
    }
    reg.push_back_line(sep);

    // rows with multi-line cell support
    for r in &rows {
        // split into columns of lines
        let cols: Vec<Vec<&str>> = r.iter().map(|c| c.split('\n').collect()).collect();
        let max_lines = cols.iter().map(|c| c.len()).max().unwrap_or(0);
        for row_i in 0..max_lines {
            let mut l = Line::new();
            l.push("|");
            for (col_i, col) in cols.iter().enumerate() {
                let cell_line = col.get(row_i).copied().unwrap_or("");
                l.push(pad_to_width(cell_line, widths[col_i], Align::Left));
                l.push(" |");
            }
            reg.push_back_line(l);
        }
    }

    println!("\n{}\n", reg.apply());

    // Nested list example using Region operations
    println!("# Nested list example");
    let mut r = Region::new();
    r.push_back_line(Line::from_str("- Fruits"));
    r.push_back_line(Line::from_str("  - Apple"));
    r.push_back_line(Line::from_str("  - Banana"));
    r.push_back_line(Line::from_str("- Vegetables"));
    r.push_back_line(Line::from_str("  - Carrot"));
    r.push_back_line(Line::from_str("  - 长茎蘑菇"));
    println!("{}", r.apply());
}

fn main() {
    print_basic_table();
    print_aligned_table();
    print_multiline_cells_and_nested_list();
}
