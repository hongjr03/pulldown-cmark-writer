use std::fmt::{self, Display, Formatter};
use std::sync::Arc;

/// A Fragment is the smallest unit: an owned, cheaply clonable piece of text.
/// Internally we use Arc<str> so cloning fragments is cheap and we avoid
/// unnecessary allocations while composing lines/regions.
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct Fragment(Arc<str>);

impl Fragment {
    /// Create a fragment from a &str
    pub fn from_str(s: &str) -> Self {
        Fragment(Arc::from(s.to_owned()))
    }

    /// Create a fragment from a String
    pub fn from_string(s: String) -> Self {
        Fragment(Arc::from(s))
    }

    /// Create a fragment which is n spaces (useful for indentation)
    pub fn spaces(n: usize) -> Self {
        Fragment::from_string(" ".repeat(n))
    }

    /// Return the inner &str
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Character length
    pub fn len(&self) -> usize {
        self.as_str().chars().count()
    }
}

impl From<&str> for Fragment {
    fn from(s: &str) -> Self {
        Fragment::from_str(s)
    }
}

impl From<String> for Fragment {
    fn from(s: String) -> Self {
        Fragment::from_string(s)
    }
}

impl Display for Fragment {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
