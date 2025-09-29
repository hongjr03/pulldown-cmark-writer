pub mod block;
pub mod custom;
pub mod inline;
pub mod parse;
pub mod writer;

pub use block::Block;
pub use block::block_to_events;
pub use inline::Inline;
pub use inline::inline_to_events;
pub use parse::parse_events_to_blocks;
pub use parse::parse_events_to_blocks_with_parsers;
pub use writer::blocks_to_markdown;

pub use custom::{BlockNode, InlineNode};

/// Context passed to a parse hook. This struct gives limited visibility into
/// the parser's current state so a hook can make context-aware decisions.
///
/// Fields:
/// - `depth`: current stack depth (0 == top-level)
/// - `parent_tag`: the parent's `Tag<'static>` (if any)
/// - `parent_collects_inlines`: whether the parent frame is collecting inlines
/// - `event_index`: current event index in the original slice
pub struct ParseContext {
    /// current stack depth (0 == top-level)
    pub depth: usize,
    /// parent's tag (if any), converted to a 'static Tag for convenience
    pub parent_tag: Option<pulldown_cmark::Tag<'static>>,
    /// whether parent frame (if any) is collecting inlines
    pub parent_collects_inlines: bool,
    /// current event index in the original slice
    pub event_index: usize,
}
