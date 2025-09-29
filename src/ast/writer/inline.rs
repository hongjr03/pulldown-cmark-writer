use crate::ast::Inline;
use crate::text::Line;

pub fn append_inline_to_line(line: &mut Line, inl: &Inline) -> Option<(String, String, String)> {
    match inl {
        Inline::Text(r) => {
            let s = r.apply();
            for (i, part) in s.split('\n').enumerate() {
                if i > 0 {
                    line.push("\n");
                }
                line.push(part);
            }
        }
        Inline::Code(r) => {
            let s = r.apply();
            let ticks = if s.contains('`') { "``" } else { "`" };
            line.push(format!("{}{}{}", ticks, s, ticks));
        }
        Inline::InlineHtml(r) | Inline::Html(r) => {
            line.push(r.apply());
        }
        Inline::SoftBreak => {
            line.push(" ");
        }
        Inline::HardBreak => {
            line.push("  \n");
        }
        Inline::Emphasis(children) => {
            line.push("*");
            for c in children {
                append_inline_to_line(line, c);
            }
            line.push("*");
        }
        Inline::Strong(children) => {
            line.push("**");
            for c in children {
                append_inline_to_line(line, c);
            }
            line.push("**");
        }
        Inline::Strikethrough(children) => {
            line.push("~~");
            for c in children {
                append_inline_to_line(line, c);
            }
            line.push("~~");
        }
        Inline::Subscript(children) => {
            line.push("~{");
            for c in children {
                append_inline_to_line(line, c);
            }
            line.push("}");
        }
        Inline::Superscript(children) => {
            line.push("^{");
            for c in children {
                append_inline_to_line(line, c);
            }
            line.push("}");
        }
        Inline::Link {
            link_type,
            dest,
            title,
            id,
            children,
        } => {
            let mut inner = String::new();
            for c in children {
                let mut tmp = Line::new();
                append_inline_to_line(&mut tmp, c);
                inner.push_str(&tmp.apply());
            }
            use pulldown_cmark::LinkType;
            match link_type {
                LinkType::Reference if !id.is_empty() => {
                    line.push(format!("[{}][{}]", inner, id));
                    return Some((id.clone(), dest.clone(), title.clone()));
                }
                LinkType::Autolink | LinkType::Email => {
                    line.push(format!("<{}>", dest));
                }
                LinkType::Shortcut | LinkType::Collapsed if !id.is_empty() => {
                    line.push(format!("[{}]", inner));
                    return Some((id.clone(), dest.clone(), title.clone()));
                }
                _ => {
                    let safe_dest = dest
                        .replace('\\', "\\\\")
                        .replace(')', "\\)")
                        .replace('(', "\\(");
                    if title.is_empty() {
                        line.push(format!("[{}]({})", inner, safe_dest));
                    } else {
                        let safe_title = title.replace('\\', "\\\\").replace('"', "\\\"");
                        line.push(format!("[{}]({} \"{}\")", inner, safe_dest, safe_title));
                    }
                }
            }
        }
        Inline::Image {
            link_type,
            dest,
            title,
            id,
            children,
        } => {
            let mut inner = String::new();
            for c in children {
                let mut tmp = Line::new();
                append_inline_to_line(&mut tmp, c);
                inner.push_str(&tmp.apply());
            }
            use pulldown_cmark::LinkType;
            match link_type {
                LinkType::Reference if !id.is_empty() => {
                    line.push(format!("![{}][{}]", inner, id));
                    return Some((id.clone(), dest.clone(), title.clone()));
                }
                LinkType::Shortcut | LinkType::Collapsed if !id.is_empty() => {
                    line.push(format!("![{}]", inner));
                    return Some((id.clone(), dest.clone(), title.clone()));
                }
                _ => {
                    if title.is_empty() {
                        line.push(format!("![{}]({})", inner, dest));
                    } else {
                        line.push(format!("![{}]({} \"{}\")", inner, dest, title));
                    }
                }
            }
        }
        Inline::FootnoteReference(s) => {
            line.push(format!("[^{}]", s));
        }
        Inline::InlineMath(r) => {
            line.push(format!("${}$", r.apply()));
        }
        Inline::DisplayMath(r) => {
            line.push("\n$$\n");
            line.push(r.apply());
            line.push("\n$$\n");
        }
    }
    None
}
