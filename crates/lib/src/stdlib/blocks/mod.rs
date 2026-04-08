mod capture_block;
mod case_block;
mod comment_block;
mod for_block;
mod if_block;
mod ifchanged_block;
mod raw_block;

pub use self::capture_block::CaptureBlock;
pub use self::case_block::CaseBlock;
pub use self::comment_block::CommentBlock;
pub use self::for_block::ForBlock;
pub use self::for_block::TableRowBlock;
pub use self::if_block::IfBlock;
pub use self::if_block::UnlessBlock;
pub use self::ifchanged_block::IfChangedBlock;
pub use self::raw_block::RawBlock;

pub use self::for_block::ForloopObject;
pub use self::for_block::Range;
pub use self::for_block::RangeExpression;

use liquid_core::Renderable;

fn elements_are_blank(elements: &[Box<dyn Renderable>]) -> bool {
    elements
        .iter()
        .all(|element| element.blankness().is_blank())
}

fn remove_blank_text_nodes(elements: &mut Vec<Box<dyn Renderable>>) {
    elements.retain(|element| !element.blankness().is_blank_text());
}
