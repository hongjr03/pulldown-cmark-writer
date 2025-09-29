use pulldown_cmark::Alignment as PAlign;
use unicode_width::UnicodeWidthStr;

pub fn pad_to_width(s: &str, width: usize, align: Option<&PAlign>) -> String {
    let w = UnicodeWidthStr::width(s);
    if width <= w {
        return s.to_string();
    }
    let pad = width - w;
    match align {
        Some(&PAlign::Left) => {
            let mut out = String::from(s);
            out.push_str(&" ".repeat(pad));
            out
        }
        Some(&PAlign::None) => {
            let mut out = String::from(s);
            out.push_str(&" ".repeat(pad));
            out
        }
        Some(&PAlign::Right) => {
            let mut out = String::new();
            out.push_str(&" ".repeat(pad));
            out.push_str(s);
            out
        }
        Some(&PAlign::Center) => {
            let left = pad / 2;
            let right = pad - left;
            let mut out = String::new();
            out.push_str(&" ".repeat(left));
            out.push_str(s);
            out.push_str(&" ".repeat(right));
            out
        }
        None => {
            let mut out = String::from(s);
            out.push_str(&" ".repeat(pad));
            out
        }
    }
}
