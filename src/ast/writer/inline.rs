use crate::ast::Inline;
use crate::text::Line;
// inline writer doesn't need the custom trait import here

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
        Inline::Custom(c_arc) => {
            let c = c_arc.clone();
            // small stack to hold link/image destinations for End handling
            let mut link_stack: Vec<(bool, String, String)> = Vec::new();
            for ev in c.to_events() {
                match ev {
                    pulldown_cmark::Event::Text(t) => {
                        line.push(t.into_string());
                    }
                    pulldown_cmark::Event::InlineHtml(t) | pulldown_cmark::Event::Html(t) => {
                        line.push(t.into_string());
                    }
                    pulldown_cmark::Event::Code(t) => {
                        let s = t.into_string();
                        let ticks = if s.contains('`') { "``" } else { "`" };
                        line.push(format!("{}{}{}", ticks, s, ticks));
                    }
                    pulldown_cmark::Event::SoftBreak => {
                        line.push(" ");
                    }
                    pulldown_cmark::Event::HardBreak => {
                        line.push("  \n");
                    }
                    pulldown_cmark::Event::Start(tag) => {
                        use pulldown_cmark::Tag;
                        match tag {
                            Tag::Emphasis => {
                                line.push("*");
                            }
                            Tag::Strong => {
                                line.push("**");
                            }
                            Tag::Strikethrough => {
                                line.push("~~");
                            }
                            Tag::Subscript => {
                                line.push("~{");
                            }
                            Tag::Superscript => {
                                line.push("^{");
                            }
                            Tag::Link {
                                link_type: _,
                                dest_url,
                                title,
                                id: _,
                            } => {
                                // open link text
                                line.push("[");
                                link_stack.push((false, dest_url.to_string(), title.to_string()));
                            }
                            Tag::Image {
                                link_type: _,
                                dest_url,
                                title,
                                id: _,
                            } => {
                                // open image text
                                line.push("![");
                                link_stack.push((true, dest_url.to_string(), title.to_string()));
                            }
                            _ => {}
                        }
                    }
                    pulldown_cmark::Event::End(tagend) => {
                        use pulldown_cmark::TagEnd;
                        match tagend {
                            TagEnd::Emphasis => {
                                line.push("*");
                            }
                            TagEnd::Strong => {
                                line.push("**");
                            }
                            TagEnd::Strikethrough => {
                                line.push("~~");
                            }
                            TagEnd::Subscript => {
                                line.push("}");
                            }
                            TagEnd::Superscript => {
                                line.push("}");
                            }
                            TagEnd::Link => {
                                if let Some((is_image, dest, title)) = link_stack.pop() {
                                    if is_image {
                                        // close image: ](dest "title")
                                        if title.is_empty() {
                                            line.push(format!("]({})", dest));
                                        } else {
                                            let safe_title = title.replace('"', "\\\"");
                                            line.push(format!("]({} \"{}\")", dest, safe_title));
                                        }
                                    } else {
                                        // close link
                                        if title.is_empty() {
                                            line.push(format!("]({})", dest));
                                        } else {
                                            let safe_title = title.replace('"', "\\\"");
                                            line.push(format!("]({} \"{}\")", dest, safe_title));
                                        }
                                    }
                                }
                            }
                            TagEnd::Image => {
                                // Image end should be handled by TagEnd::Link branch when pushed as Image start
                                if let Some((is_image, dest, title)) = link_stack.pop() {
                                    if is_image {
                                        if title.is_empty() {
                                            line.push(format!("]({})", dest));
                                        } else {
                                            let safe_title = title.replace('"', "\\\"");
                                            line.push(format!("]({} \"{}\")", dest, safe_title));
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    None
}
