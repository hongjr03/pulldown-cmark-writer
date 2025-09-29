use super::{Fragment, Line};
use std::fmt::{self, Display, Formatter};

/// A Region is a 2D collection of lines. We provide chainable operations that
/// mutate the region in-place and return &mut Self so callers can chain many
/// operations without repeatedly reallocating strings.
#[derive(Clone, Debug, Default)]
pub struct Region {
    lines: Vec<Line>,
    // optional suffix lines that are logically appended after the main
    // region. This is used to carry things like reference-definition
    // blocks that should be emitted after the region's main content, and
    // which must participate in prefixing/indentation when the region is
    // transformed (for example, reference defs inside a blockquote need
    // to be quoted as well).
    suffix: Vec<Line>,
}

impl Region {
    pub fn new() -> Self {
        Region {
            lines: Vec::new(),
            suffix: Vec::new(),
        }
    }

    /// Create a region from a multiline &str (split on "\n")
    pub fn from_str(s: &str) -> Self {
        let lines = if s.is_empty() {
            Vec::new()
        } else {
            s.split('\n').map(|l| Line::from_str(l)).collect()
        };
        Region {
            lines,
            suffix: Vec::new(),
        }
    }

    /// Push a line to the front
    pub fn push_front_line(&mut self, line: Line) -> &mut Self {
        self.lines.insert(0, line);
        self
    }

    /// Push a line to the back
    pub fn push_back_line(&mut self, line: Line) -> &mut Self {
        self.lines.push(line);
        self
    }

    /// Push a line to the suffix (appended after the main lines)
    pub fn push_back_suffix_line(&mut self, line: Line) -> &mut Self {
        self.suffix.push(line);
        self
    }

    /// Add a prefix fragment to every line
    pub fn prefix_each_line<F: Into<Fragment>>(&mut self, prefix: F) -> &mut Self {
        let p = prefix.into();
        for line in &mut self.lines {
            line.prepend(p.clone());
        }
        for line in &mut self.suffix {
            line.prepend(p.clone());
        }
        self
    }

    /// Indent each line by `n` spaces
    pub fn indent_each_line(&mut self, n: usize) -> &mut Self {
        if n == 0 {
            return self;
        }
        let sp = Fragment::spaces(n);
        for line in &mut self.lines {
            line.prepend(sp.clone());
        }
        for line in &mut self.suffix {
            line.prepend(sp.clone());
        }
        self
    }

    /// Add a prefix to the first line, and for the remaining lines add equal
    /// amount of spaces so they line up with the remainder of the first line.
    /// For example: prefix_first_then_indent_rest("- ") on ["a","b"] ->
    /// ["- a","  b"]. The spaces count is based on the prefix's char length.
    pub fn prefix_first_then_indent_rest<F: Into<Fragment>>(&mut self, prefix: F) -> &mut Self {
        let p = prefix.into();
        let pad = p.len();
        if let Some(first) = self.lines.get_mut(0) {
            first.prepend(p.clone());
        } else if let Some(first) = self.suffix.get_mut(0) {
            // if main lines are empty, apply prefix to first suffix line
            first.prepend(p.clone());
        }
        if pad > 0 {
            let sp = Fragment::spaces(pad);
            for line in self.lines.iter_mut().skip(1) {
                line.prepend(sp.clone());
            }
            // indent the suffix lines as well
            for line in self.suffix.iter_mut() {
                line.prepend(sp.clone());
            }
        }
        self
    }

    /// Convert the region into a String, joining lines with '\n'. This is the
    /// only place we eagerly allocate the final result.
    pub fn apply(&self) -> String {
        let mut out = String::new();
        let mut first = true;
        for line in &self.lines {
            if !first {
                out.push('\n');
            }
            out.push_str(&line.apply());
            first = false;
        }
        for line in &self.suffix {
            if !first {
                out.push('\n');
            }
            out.push_str(&line.apply());
            first = false;
        }
        out
    }

    /// Convenience to check whether region is empty
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty() && self.suffix.is_empty()
    }

    /// Consume the Region and return its lines as a Vec<Line>.
    pub fn into_lines(self) -> Vec<Line> {
        let mut out = self.lines;
        out.extend(self.suffix);
        out
    }
}

impl Display for Region {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&self.apply())
    }
}
