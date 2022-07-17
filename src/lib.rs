pub mod error;
pub mod item;
pub mod numbers;

use error::*;
use item::*;
use numbers::*;

pub type Scalar = i16;

#[derive(Debug, Clone)]
struct ItemWithCalcSize<const ND: usize> {
    item: Item<ND>,
    position: [Scalar; ND],
    size: [Scalar; ND],
}

impl<const ND: usize> Default for ItemWithCalcSize<ND> {
    fn default() -> Self {
        Self {
            item: Default::default(),
            position: [Default::default(); ND],
            size: [Default::default(); ND],
        }
    }
}

#[derive(Debug, Clone)]
struct ExtentAndMargins<const ND: usize> {
    pub margin_start: Scalar,
    pub extent: Scalar,
    pub margin_end: Scalar,
    pub flags: ItemFlags<ND>,
}

#[derive(Debug, Clone)]
pub struct Context<const ND: usize> {
    last_id: Id,
    pub(crate) items: std::collections::BTreeMap<Id, ItemWithCalcSize<ND>>,
}

impl<const ND: usize> Default for Context<ND> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const ND: usize> Context<ND> {
    pub fn new() -> Self {
        let items = Default::default();
        Self {
            last_id: Id(0),
            items,
        }
    }

    // ======
    // run layout algorithm
    // vvvvvv
    pub fn run() {
        unimplemented!()
    }
    pub fn run_item(&mut self, item_id: Id) -> Result<(), ItemNotFound> {
        for i in 0..ND {
            self.calc_size(item_id, i.try_into().unwrap())?;
        }
        for i in 0..ND {
            self.arrange(item_id, i.try_into().unwrap())?;
        }
        Ok(())
    }

    fn calc_size(&mut self, item_id: Id, dim: Fin<ND>) -> Result<(), ItemNotFound> {
        // recursively call calc_size
        let parent = self.item_err(item_id)?;
        let mut maybe_r = parent.first_child;
        while let Some(r_id) = maybe_r {
            self.calc_size(r_id, dim)?;
            let r_item = self.item_err(r_id)?;
            maybe_r = r_item.next_sibling;
        }

        let ItemWithCalcSize {
            item,
            position: rect_pos,
            size: rect_size,
        } = self.item_rect_mut_err(item_id)?;

        // early return if size specified by user
        if let Some(size) = item.size[dim.into_usize()] {
            rect_pos[dim.into_usize()] = item.margins[dim.into_usize()].start;
            rect_size[dim.into_usize()] = size;
            return Ok(());
        }
        let flags_as_parent = item.flags.as_parent.clone();

        let calc_size = match flags_as_parent.layout {
            Layout::Fixed => self.calc_cross_axis(item_id, dim, false)?,
            Layout::Flex(along_dim) => {
                if dim == along_dim {
                    self.calc_along_axis(item_id, dim, flags_as_parent.allow_wrap)?
                } else {
                    self.calc_cross_axis(item_id, dim, flags_as_parent.allow_wrap)?
                }
            }
        };

        // dance with borrow checker
        let ItemWithCalcSize {
            item,
            position: rect_pos,
            size: rect_size,
        } = self.item_rect_mut_err(item_id)?;
        rect_pos[dim.into_usize()] = item.margins[dim.into_usize()].start;
        rect_size[dim.into_usize()] = calc_size;

        Ok(())
    }

    fn calc_extent_margins(
        &mut self,
        parent_id: Id,
        dim: Fin<ND>,
        mut callback: impl FnMut(ExtentAndMargins<ND>) -> Result<(), ItemNotFound>,
    ) -> Result<(), ItemNotFound> {
        self.foreach_mut_children_rect(
            parent_id,
            |ItemWithCalcSize {
                 item,
                 position: _,
                 size: rect_size,
             }| {
                let r = ExtentAndMargins {
                    flags: item.flags.clone(),
                    margin_start: item.margins[dim.into_usize()].start,
                    extent: rect_size[dim.into_usize()],
                    margin_end: item.margins[dim.into_usize()].end,
                };
                callback(r)
            },
        )
    }

    /// [ a ]
    /// [  b   ]
    /// [      c       ]
    /// [   d   ]
    /// <-- measure --->
    fn calc_cross_axis(
        &mut self,
        item_id: Id,
        dim: Fin<ND>,
        respect_line_break: bool,
    ) -> Result<Scalar, ItemNotFound> {
        let mut max_size = 0;
        let mut hist_acc_size = 0;
        self.calc_extent_margins(item_id, dim, |xx| {
            if respect_line_break && xx.flags.as_child.wrap_me {
                hist_acc_size += max_size;
                max_size = 0;
            }
            // IMPROVE: return ExtentAndMargins, not Scalar
            // align left or right has a different
            max_size = Scalar::max(max_size, xx.margin_start + xx.extent + xx.margin_end);
            Ok(())
        })?;
        Ok(max_size + hist_acc_size)
    }

    /// [ a ] [b] [  c ]
    /// <-- measure --->
    fn calc_along_axis(
        &mut self,
        item_id: Id,
        dim: Fin<ND>,
        respect_line_break: bool,
    ) -> Result<Scalar, ItemNotFound> {
        let mut acc_size = 0;
        let mut hist_max_size = 0;
        let mut last_margin_end = 0;
        self.calc_extent_margins(item_id, dim, |xx| {
            if respect_line_break && xx.flags.as_child.wrap_me {
                hist_max_size = Scalar::max(acc_size, hist_max_size);
                acc_size = 0;
            }
            acc_size += Scalar::max(last_margin_end, xx.margin_start) + xx.extent + xx.margin_end;
            last_margin_end = xx.margin_end;
            Ok(())
        })?;
        Ok(Scalar::max(acc_size, hist_max_size))
    }

    fn arrange(&mut self, item_id: Id, dim: Fin<ND>) -> Result<(), ItemNotFound> {
        unimplemented!()
    }

    // ======
    // adding item
    // vvvvvv

    /// add new item with no parent and no children
    pub fn item_new(&mut self) -> &Id {
        self.last_id.0 += 1;
        self.items.insert(self.last_id, Default::default());
        &self.last_id
    }

    // ======
    // accessing
    // vvvvvv

    pub fn items_count(&self) -> usize {
        self.items.len()
    }

    pub fn item(&self, item_id: Id) -> Option<&Item<ND>> {
        self.items.get(&item_id).map(|x| &x.item)
    }

    pub fn item_mut(&mut self, item_id: Id) -> Option<&mut Item<ND>> {
        self.items.get_mut(&item_id).map(|x| &mut x.item)
    }

    fn item_rect_err(&self, item_id: Id) -> Result<&ItemWithCalcSize<ND>, ItemNotFound> {
        match self.items.get(&item_id) {
            Some(x) => Ok(x),
            None => Err(ItemNotFound(item_id)),
        }
    }

    fn item_rect_mut_err(
        &mut self,
        item_id: Id,
    ) -> Result<&mut ItemWithCalcSize<ND>, ItemNotFound> {
        match self.items.get_mut(&item_id) {
            Some(x) => Ok(x),
            None => Err(ItemNotFound(item_id)),
        }
    }

    /// Error version of `self.item`
    pub fn item_err(&self, item_id: Id) -> Result<&Item<ND>, ItemNotFound> {
        match self.item(item_id) {
            Some(x) => Ok(x),
            None => Err(ItemNotFound(item_id)),
        }
    }

    /// Error version of `self.item_mut`
    pub fn item_mut_err(&mut self, item_id: Id) -> Result<&mut Item<ND>, ItemNotFound> {
        match self.item_mut(item_id) {
            Some(x) => Ok(x),
            None => Err(ItemNotFound(item_id)),
        }
    }

    // pub fn iter_children(
    fn foreach_mut_children_rect(
        &mut self,
        parent_id: Id,
        mut callback: impl FnMut(&mut ItemWithCalcSize<ND>) -> Result<(), ItemNotFound>,
    ) -> Result<(), ItemNotFound> {
        let parent = self.item_err(parent_id)?;
        let mut maybe_child_id = parent.first_child;

        while let Some(child_id) = maybe_child_id {
            let current_child = self.item_rect_mut_err(child_id)?;
            callback(current_child)?;
            maybe_child_id = current_child.item.next_sibling;
        }
        Ok(())
    }

    pub fn item_mut_last_child(
        &mut self,
        parent_id: Id,
    ) -> Result<Option<&mut Item<ND>>, ItemNotFound> {
        let mut last_child: Option<*mut Item<ND>> = None;
        self.foreach_mut_children_rect(parent_id, |xx| {
            last_child = Some(&mut xx.item);
            Ok(())
        })?;
        Ok(last_child.map(|p| unsafe { &mut *p }))
    }

    // ======
    // Changing hiearchy
    // vvvvvv

    /// insert `later` item as next sibling of `earlier` item
    pub fn insert_after(&mut self, earlier_id: Id, later_id: Id) -> Result<(), ItemNotFound> {
        let earlier_next_sibling = self.item_err(earlier_id)?.next_sibling;
        let later = self.item_mut_err(later_id)?;
        later.next_sibling = earlier_next_sibling;
        later.flags.extra.inserted = true;
        let earlier = self.item_mut_err(earlier_id)?;
        earlier.next_sibling = Some(later_id);
        Ok(())
    }

    /// set parent of an item, then insert as first child
    pub fn push_front(&mut self, parent_id: Id, child_id: Id) -> Result<(), ItemNotFound> {
        // existence check
        let _child = self.item_mut_err(child_id)?;
        let parent = self.item_mut_err(parent_id)?;
        match parent.first_child {
            Some(previous_first_child_id) => {
                parent.first_child = Some(child_id);
                let child = self.item_mut_err(child_id)?;
                child.next_sibling = Some(previous_first_child_id);
            }
            None => {
                parent.first_child = Some(child_id);
            }
        }
        Ok(())
    }

    /// set parent of an item, then insert as last child
    pub fn push_back(&mut self, parent_id: Id, child_id: Id) -> Result<(), ItemNotFound> {
        // existence check
        let _child = self.item_mut_err(child_id)?;
        match self.item_mut_last_child(parent_id)? {
            Some(last_child) => {
                last_child.next_sibling = Some(child_id);
            }
            None => {
                let parent = self.item_mut_err(parent_id)?;
                parent.first_child = Some(child_id);
            }
        }
        Ok(())
    }
}
