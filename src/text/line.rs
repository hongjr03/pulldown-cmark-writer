use super::Fragment;
use std::fmt::{self, Display, Formatter};

/// A Line is a sequence of Fragments. We avoid joining fragments until the
/// final `apply()` so intermediate operations can cheaply clone fragments.
#[derive(Clone, Debug, Default)]
pub struct Line {
    fragments: Vec<Fragment>,
}

impl Line {
    pub fn new() -> Self {
        Line {
            fragments: Vec::new(),
        }
    }

    /// Create a line with a single fragment from &str
    pub fn from_str(s: &str) -> Self {
        Line {
            fragments: vec![Fragment::from(s)],
        }
    }

    /// Push fragment to the end
    pub fn push<F: Into<Fragment>>(&mut self, f: F) -> &mut Self {
        self.fragments.push(f.into());
        self
    }

    /// Prepend a fragment to the start of the line
    pub fn prepend<F: Into<Fragment>>(&mut self, f: F) -> &mut Self {
        self.fragments.insert(0, f.into());
        self
    }

    /// Join fragments into a single String
    pub fn apply(&self) -> String {
        let mut out = String::new();
        for frag in &self.fragments {
            out.push_str(frag.as_str());
        }
        out
    }
}

impl Display for Line {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&self.apply())
    }
}
