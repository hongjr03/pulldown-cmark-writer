//! User-custom nodes support.
//!
//! This module defines traits that allow consumers to plug in custom
//! inline/block nodes. We provide default marker types so the library
//! remains compatible when the user doesn't supply custom nodes.

use crate::{Line, Region};
use pulldown_cmark::Event;

/// Trait describing a user-defined block node.
///
/// Implementors should provide a way to convert their node into a sequence
/// of pulldown-cmark `Event<'static>` values for serialization.
pub trait BlockNode: std::fmt::Debug + Send + Sync {
    /// Convert the custom block into owned pulldown-cmark events.
    fn to_events(&self) -> Vec<Event<'static>>;
    /// Provide a direct rendering of this block as a `Region`.
    /// The writer will use this `Region` directly when
    /// converting blocks to markdown.
    fn to_region(&self) -> Region;
}

/// Trait describing a user-defined inline node.
pub trait InlineNode: std::fmt::Debug + Send + Sync {
    /// Convert the custom inline into owned pulldown-cmark events.
    fn to_events(&self) -> Vec<Event<'static>>;
    /// Provide a direct rendering of this inline as a `Line`.
    /// The writer will use this `Line` directly when
    /// converting inlines to markdown.
    fn to_line(&self) -> Line;
}

/// Optional trait that allows consumers to provide a parser for custom
/// block nodes. Implementors should decide whether the events at the
/// current position match their node and return the number of consumed
/// events along with a constructed `Block` when they do.
pub trait BlockParser: Send + Sync {
    fn try_parse(
        &self,
        events: &[Event],
        idx: usize,
        ctx: &crate::ast::ParseContext,
    ) -> Option<(usize, crate::ast::Block)>;
}

/// Default empty marker for when no custom block node is used.
#[derive(Clone, Debug)]
pub struct NoBlock;
impl BlockNode for NoBlock {
    fn to_events(&self) -> Vec<Event<'static>> {
        Vec::new()
    }
    fn to_region(&self) -> Region {
        Region::new()
    }
}

/// Default empty marker for when no custom inline node is used.
#[derive(Clone, Debug)]
pub struct NoInline;
impl InlineNode for NoInline {
    fn to_events(&self) -> Vec<Event<'static>> {
        Vec::new()
    }
    fn to_line(&self) -> Line {
        Line::new()
    }
}
