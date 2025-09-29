use crate::ast::Inline;
use crate::text::Line;

/// A small type representing a reference-style link/image definition.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReferenceDef {
    pub id: String,
    pub dest: String,
    pub title: String,
}

/// Produce a Line for the provided `Inline` and optionally return a
/// reference-definition tuple when the inline was a reference-style link/image.
pub fn inline_to_line(inl: &Inline) -> (Line, Option<ReferenceDef>) {
    let mut line = Line::new();
    let mut def: Option<ReferenceDef> = None;
    match inl {
        Inline::Text(r) => {
            let lines = r.lines();
            for (i, ln) in lines.iter().enumerate() {
                if i > 0 {
                    line.push("\n");
                }
                line.extend_from_line(ln);
            }
        }
        Inline::Code(r) => {
            let s = r.apply();
            let ticks = if s.contains('`') { "``" } else { "`" };
            line.push(format!("{}{}{}", ticks, s, ticks));
        }
        Inline::InlineHtml(r) | Inline::Html(r) => {
            let lines = r.lines();
            for (i, ln) in lines.iter().enumerate() {
                if i > 0 {
                    line.push("\n");
                }
                line.extend_from_line(ln);
            }
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
                let (ln, d) = inline_to_line(c);
                line.extend_from_line(&ln);
                if def.is_none() {
                    def = d;
                }
            }
            line.push("*");
        }
        Inline::Strong(children) => {
            line.push("**");
            for c in children {
                let (ln, d) = inline_to_line(c);
                line.extend_from_line(&ln);
                if def.is_none() {
                    def = d;
                }
            }
            line.push("**");
        }
        Inline::Strikethrough(children) => {
            line.push("~~");
            for c in children {
                let (ln, d) = inline_to_line(c);
                line.extend_from_line(&ln);
                if def.is_none() {
                    def = d;
                }
            }
            line.push("~~");
        }
        Inline::Subscript(children) => {
            line.push("~{");
            for c in children {
                let (ln, d) = inline_to_line(c);
                line.extend_from_line(&ln);
                if def.is_none() {
                    def = d;
                }
            }
            line.push("}");
        }
        Inline::Superscript(children) => {
            line.push("^{");
            for c in children {
                let (ln, d) = inline_to_line(c);
                line.extend_from_line(&ln);
                if def.is_none() {
                    def = d;
                }
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
            let mut inner = Line::new();
            for c in children {
                let (ln, d) = inline_to_line(c);
                inner.extend_from_line(&ln);
                if def.is_none() {
                    def = d;
                }
            }
            use pulldown_cmark::LinkType;
            match link_type {
                LinkType::Reference if !id.is_empty() => {
                    line.push(format!("[{}][{}]", inner.apply(), id));
                    def = Some(ReferenceDef {
                        id: id.clone(),
                        dest: dest.clone(),
                        title: title.clone(),
                    });
                }
                LinkType::Autolink | LinkType::Email => {
                    line.push(format!("<{}>", dest));
                }
                LinkType::Shortcut | LinkType::Collapsed if !id.is_empty() => {
                    line.push(format!("[{}]", inner.apply()));
                    def = Some(ReferenceDef {
                        id: id.clone(),
                        dest: dest.clone(),
                        title: title.clone(),
                    });
                }
                _ => {
                    let safe_dest = dest
                        .replace('\\', "\\\\")
                        .replace(')', "\\)")
                        .replace('(', "\\(");
                    if title.is_empty() {
                        line.push(format!("[{}]({})", inner.apply(), safe_dest));
                    } else {
                        let safe_title = title.replace('\\', "\\\\").replace('"', "\\\"");
                        line.push(format!(
                            "[{}]({} \"{}\")",
                            inner.apply(),
                            safe_dest,
                            safe_title
                        ));
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
            let mut inner = Line::new();
            for c in children {
                let (ln, d) = inline_to_line(c);
                inner.extend_from_line(&ln);
                if def.is_none() {
                    def = d;
                }
            }
            use pulldown_cmark::LinkType;
            match link_type {
                LinkType::Reference if !id.is_empty() => {
                    line.push(format!("![{}][{}]", inner.apply(), id));
                    def = Some(ReferenceDef {
                        id: id.clone(),
                        dest: dest.clone(),
                        title: title.clone(),
                    });
                }
                LinkType::Shortcut | LinkType::Collapsed if !id.is_empty() => {
                    line.push(format!("![{}]", inner.apply()));
                    def = Some(ReferenceDef {
                        id: id.clone(),
                        dest: dest.clone(),
                        title: title.clone(),
                    });
                }
                _ => {
                    if title.is_empty() {
                        line.push(format!("![{}]({})", inner.apply(), dest));
                    } else {
                        line.push(format!("![{}]({} \"{}\")", inner.apply(), dest, title));
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
        Inline::Custom(c) => {
            line.push(c.to_line().apply());
        }
    }
    (line, def)
}
