use crate::Fin;
use crate::Scalar;

/// todo: add guard or lifetime constraint to make this only usable in 1 Context
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Id(pub(crate) u32);

#[derive(Debug, Default, Clone, Copy)]
pub struct Margin {
    pub start: Scalar,
    pub end: Scalar,
}

/// Mostly a rectangle
#[derive(Debug, Clone)]
pub struct Item<const ND: usize> {
    pub flags: ItemFlags<ND>,
    /// ltrb
    pub margins: [Margin; ND],
    /// Leave None to automatically get size
    pub size: [Option<Scalar>; ND],
    pub(crate) first_child: Option<Id>,
    pub(crate) next_sibling: Option<Id>,
}

impl<const ND: usize> Default for Item<ND> {
    fn default() -> Self {
        Self {
            flags: Default::default(),
            margins: [Default::default(); ND],
            size: [None; ND],
            first_child: None,
            next_sibling: None,
        }
    }
}

#[derive(Default, Clone, Debug)]
pub struct ItemFlags<const ND: usize> {
    pub as_parent: AsParentFlags<ND>,
    pub as_child: AsChildFlags<ND>,
    pub extra: ExtraFlags<ND>,
}

#[derive(Default, Clone, Debug)]
pub enum Layout<const ND: usize> {
    #[default]
    Fixed,
    Flex(Fin<ND>),
}

// impl Layout
// pub const FlexRow = Self::Flex(0);

/// flags for being as container for other rectangles
/// control layout of children
#[derive(Clone, Debug)]
pub struct AsParentFlags<const ND: usize> {
    /// use flex model or not
    pub layout: Layout<ND>,
    /// wrap around line when child request that
    pub allow_wrap: bool,
    /// wrap around line when line too long
    pub auto_wrap: bool,
    /// start
    /// |[ T ][ E ][ X ][ T ]      |
    /// end
    /// |      [ T ][ E ][ X ][ T ]|
    /// center
    /// |   [ T ][ E ][ X ][ T ]   |
    /// justify
    /// |[ T ]  [ E ]  [ X ]  [ T ]|
    pub alignment_along_axis: Alignment,
}

impl<const ND: usize> Default for AsParentFlags<ND> {
    fn default() -> Self {
        Self {
            layout: Default::default(),
            allow_wrap: true,
            auto_wrap: true,
            alignment_along_axis: Default::default(),
        }
    }
}

#[derive(Clone, Debug, Copy)]
pub struct Alignment {
    /// if this child will be anchored to the front of parent
    /// | parent       |
    /// | child |
    pub front: bool,
    /// if this child will be anchored to the back of parent
    /// | parent       |
    ///        | child |
    pub back: bool,
}

impl Default for Alignment {
    fn default() -> Self {
        Self {
            front: true,
            back: false,
        }
    }
}

/// flags for being as child
#[derive(Clone, Debug)]
pub struct AsChildFlags<const ND: usize> {
    /// | -----        -----        |
    /// | | S |        |   |        |
    /// | -----        |   |  ----- |
    /// |              |   |  | C | |
    /// |       -----  |   |  ----- |
    /// |       | E |  | J |        |
    /// |       -----  -----        |
    ///
    /// note: in the dimension along axis, this value is ignored
    pub alignment_cross_axis: [Alignment; ND],
    /// whether it'll be wrapped to be start of a new line
    pub wrap_me: bool,
}

impl<const ND: usize> Default for AsChildFlags<ND> {
    fn default() -> Self {
        Self {
            alignment_cross_axis: [Default::default(); ND],
            wrap_me: Default::default(),
        }
    }
}

#[derive(Default, Clone, Debug)]
pub struct ExtraFlags<const ND: usize> {
    /// item has been inserted (bit 10)
    pub inserted: bool,
    // /// size has been explicitly set (bit 11)
    // pub fixed: [bool; ND],
}
