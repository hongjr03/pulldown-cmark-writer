pub mod fragment;
pub mod line;
pub mod region;

pub use fragment::Fragment;
pub use line::Line;
pub use region::Region;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_apply_and_fragments() {
        let mut l = Line::new();
        l.push("hello").push(" ").push("world");
        assert_eq!(l.apply(), "hello world");
    }

    #[test]
    fn region_prefix_and_indent() {
        let mut r = Region::from_str("one\ntwo\nthree");
        r.prefix_each_line("* ");
        assert_eq!(r.apply(), "* one\n* two\n* three");

        r = Region::from_str("a\nb");
        r.prefix_first_then_indent_rest("- ");
        assert_eq!(r.apply(), "- a\n  b");
    }

    #[test]
    fn push_front_and_back() {
        let mut r = Region::new();
        r.push_back_line(Line::from_str("tail"));
        r.push_front_line(Line::from_str("head"));
        assert_eq!(r.apply(), "head\ntail");
    }
}
