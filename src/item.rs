use crate::Alignment;
use crate::Vec2;
use crate::Vec4;

/// todo: add guard or lifetime constraint to make this only usable in 1 Context
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Id(pub(crate) u32);

/// Mostly a rectangle
#[derive(Debug)]
pub struct Item {
    pub flags: ItemFlags,
    pub margins: Vec4,
    pub size: Vec2,
    pub(crate) first_child: Option<Id>,
    pub(crate) next_sibling: Option<Id>,
}

impl Item {
    pub fn clear_item_break(&mut self) {
        self.flags.as_child.break_line = false;
    }
}

impl Default for Item {
    fn default() -> Self {
        Self {
            flags: Default::default(),
            margins: Default::default(),
            size: Default::default(),
            first_child: None,
            next_sibling: None,
        }
    }
}

#[derive(Default, Clone, Debug)]
pub struct ItemFlags {
    pub as_parent: AsParentFlags,
    pub as_child: AsChildFlags,
    pub extra: ExtraFlags,
}

#[derive(Default, Clone, Debug)]
pub enum Layout {
    #[default]
    Fixed,
    FlexRow,
    FlexColumn,
}

/// flags for being as parent
/// control layout of children
#[derive(Default, Clone, Debug)]
pub struct AsParentFlags {
    /// use flex model or not
    pub layout: Layout,
    /// wrap around line or not
    pub wrap_line: bool,
    pub alignment: Alignment,
}

/// flags for being as child
#[derive(Default, Clone, Debug)]
pub struct AsChildFlags {
    /// anchor to parent's left side
    pub left: bool,
    /// anchor to parent's right side
    pub right: bool,
    /// anchor to parent's top side
    pub top: bool,
    /// anchor to parent's bottom side
    pub bottom: bool,

    pub break_line: bool,
}

#[derive(Default, Clone, Debug)]
pub struct ExtraFlags {
    /// item has been inserted (bit 10)
    pub inserted: bool,
    /// horizontal size has been explicitly set (bit 11)
    pub hfixed: bool,
    /// vertical size has been explicitly set (bit 12)
    pub vfixed: bool,
}
