#![feature(let_else)]

pub mod error;
pub mod item;
pub mod numbers;

use error::*;
use item::*;
use numbers::*;

pub type Scalar = i16;

#[derive(Debug, Clone)]
pub struct ItemWithCalcSize<const ND: usize> {
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

    /// Layout a directed tree of items
    /// 
    /// Make sure you don't have cyclic reference (DAG is fine), or the program will hang
    /// 
    ///     item_id  root of the tree
    pub fn layout_item_recursively(&mut self, item_id: Id) -> Result<(), ItemNotFound> {
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

        let xx = self.item_rect_mut_err(item_id)?;
        let size = &mut xx.size[dim.into_usize()];
        let flags_as_parent = xx.item.flags.as_parent.clone();

        // early return if size specified by user
        if let Some(user_size) = xx.item.size[dim.into_usize()] {
            *size = user_size;
            return Ok(());
        }

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
        let xx = self.item_rect_mut_err(item_id)?;
        let size = &mut xx.size[dim.into_usize()];
        *size = calc_size;

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
        let item = self.item_err(item_id)?;
        let flags_as_parent = item.flags.as_parent.clone();

        // layout direct children
        match flags_as_parent.layout {
            Layout::Fixed => self.arrange_cross_axis(item_id, dim, false)?,
            Layout::Flex(along_dim) => {
                if dim == along_dim {
                    self.arrange_along_axis(
                        item_id,
                        dim,
                        flags_as_parent.allow_wrap,
                        flags_as_parent.allow_wrap,
                    )?;
                } else {
                    self.arrange_cross_axis(item_id, dim, flags_as_parent.allow_wrap)?;
                }
            }
        }

        // recursive call to layout children's children and so on
        let item = self.item_err(item_id)?;
        let mut maybe_r = item.first_child;
        while let Some(r_id) = maybe_r {
            self.arrange(r_id, dim)?;
            let r_item = self.item_err(r_id)?;
            maybe_r = r_item.next_sibling;
        }

        Ok(())
    }

    fn arrange_along_axis(
        &mut self,
        item_id: Id,
        dim: Fin<ND>,
        allow_wrap: bool,
        auto_wrap: bool,
    ) -> Result<(), ItemNotFound> {
        let pxx = self.item_rect_mut_err(item_id)?;
        let offset = pxx.position[dim.into_usize()];
        let space = pxx.size[dim.into_usize()];
        let alignment = pxx.item.flags.as_parent.alignment_along_axis;

        let mut current_child_id = pxx.item.first_child;
        let mut gaps_before: Vec<(Id, Scalar)> = vec![];

        while current_child_id.is_some() {
            let line_start = current_child_id;
            let mut items_on_this_line = 0;
            let mut acc_line_size = 0;
            let mut last_margin_end = 0;

            'arrange_one_line: while let Some(child_id) = current_child_id {
                let xx = self.item_rect_mut_err(child_id)?;
                let item = &mut xx.item;
                let size = xx.size[dim.into_usize()];
                let margin = item.margins[dim.into_usize()].clone();
                let next_sibling_id = item.next_sibling;

                let first_in_line = line_start != current_child_id;
                let break_early = acc_line_size + last_margin_end > space;
                if ((allow_wrap && item.flags.as_child.wrap_me) || (auto_wrap && break_early))
                    && first_in_line
                {
                    acc_line_size += last_margin_end;
                    break 'arrange_one_line;
                }
                items_on_this_line += 1;
                let min_inner_margin = Scalar::max(last_margin_end, margin.start);
                gaps_before.push((child_id, min_inner_margin));
                acc_line_size += min_inner_margin;
                last_margin_end = margin.end;
                acc_line_size += size;

                current_child_id = next_sibling_id;
            }

            if items_on_this_line > 0 {
                let extra_space = space - acc_line_size;

                match (alignment.front, alignment.back) {
                    // start
                    (true, false) => {
                        // extra space in the end, do nothing
                    }
                    // center
                    (false, false) => {
                        gaps_before[0].1 += extra_space / 2;
                    }
                    // end
                    (false, true) => {
                        // space all in front
                        gaps_before[0].1 += extra_space;
                    }
                    // space-between
                    (true, true) => {
                        enlarge_gaps_inbetween(&mut gaps_before, extra_space);
                    }
                }
                self.arrange_along_axis_range(dim, offset, &gaps_before)?;
            }
        }

        Ok(())
    }

    fn arrange_along_axis_range(
        &mut self,
        dim: Fin<ND>,
        mut offset: Scalar,
        gaps_before: &[(Id, Scalar)],
    ) -> Result<(), ItemNotFound> {
        if gaps_before.is_empty() {
            Ok(())
        } else {
            let (item_id, gap_before) = gaps_before[0];
            offset += gap_before;

            let xx = self.item_rect_mut_err(item_id)?;
            let position = &mut xx.position[dim.into_usize()];
            let size = &mut xx.size[dim.into_usize()];

            *position = offset;
            offset += *size;
            self.arrange_along_axis_range(dim, offset, &gaps_before[1..])
        }
    }

    fn arrange_cross_axis(
        &mut self,
        item_id: Id,
        dim: Fin<ND>,
        allow_wrap: bool,
    ) -> Result<(), ItemNotFound> {
        let pxx = self.item_rect_mut_err(item_id)?;
        let offset = pxx.position[dim.into_usize()];
        let space = pxx.size[dim.into_usize()];
        let mut current_child_id = pxx.item.first_child;

        // if not allowed to wrap, then process all children in one go without backtracking
        if !allow_wrap {
            return self.arrange_cross_axis_range(dim, current_child_id, None, offset, space);
        }

        let mut acc_cross_axis_size = 0;

        while current_child_id.is_some() {
            let line_start = current_child_id;
            // current column's width
            let mut max_cross_axis_size = 0;

            'arrange_one_line: while let Some(child_id) = current_child_id {
                let xx = self.item_rect_mut_err(child_id)?;
                let item = &mut xx.item;
                let size = &mut xx.size[dim.into_usize()];
                let margin = item.margins[dim.into_usize()].clone();
                let size_with_margin = *size + margin.start + margin.end;
                let next_sibling_id = item.next_sibling;

                if line_start != current_child_id && item.flags.as_child.wrap_me {
                    break 'arrange_one_line;
                }

                max_cross_axis_size = Scalar::max(max_cross_axis_size, size_with_margin);

                current_child_id = next_sibling_id;
            }

            self.arrange_cross_axis_range(
                dim,
                line_start,
                current_child_id,
                offset + acc_cross_axis_size,
                max_cross_axis_size,
            )?;
            acc_cross_axis_size += max_cross_axis_size;
        }

        let pxx = self.item_rect_mut_err(item_id)?;
        let space = &mut pxx.size[dim.into_usize()];
        // TODO: is this really necessary?
        *space = acc_cross_axis_size;

        Ok(())
    }

    fn arrange_cross_axis_range(
        &mut self,
        dim: Fin<ND>,
        maybe_start_item_id: Option<Id>,
        end_before_id: Option<Id>,
        offset: Scalar,
        space: Scalar,
    ) -> Result<(), ItemNotFound> {
        if maybe_start_item_id == end_before_id {
            return Ok(());
        }

        let Some(item_id) = maybe_start_item_id else { return Ok(()) };
        let xx = self.item_rect_mut_err(item_id)?;
        let item = &mut xx.item;
        let position = &mut xx.position[dim.into_usize()];
        let size = &mut xx.size[dim.into_usize()];
        let alignment = item.flags.as_child.alignment_cross_axis[dim.into_usize()].clone();
        let margin = item.margins[dim.into_usize()].clone();
        let next_sibling_id = item.next_sibling;

        let max_size = Scalar::max(0, space - margin.start - margin.end);

        match (alignment.front, alignment.back) {
            // start
            (true, false) => {
                // IMPROVE: resize parent when remaining space too small
                // or error out
                *size = Scalar::min(*size, max_size);
                *position = margin.start;
            }
            // center
            (false, false) => {
                *size = Scalar::min(*size, max_size);
                *position = (space - *size) / 2;
            }
            // end
            (false, true) => {
                *size = Scalar::min(*size, max_size);
                *position = space - *size - margin.end;
            }
            // fill
            (true, true) => {
                *size = max_size;
                *position = margin.start;
            }
        }

        *position += offset;

        self.arrange_cross_axis_range(dim, next_sibling_id, end_before_id, offset, space)
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

    /// add new item with no parent and no children
    pub fn item_new_mut(&mut self, before_insert: impl FnOnce(&mut Item<ND>)) -> &Id {
        self.last_id.0 += 1;
        let mut xx: ItemWithCalcSize<ND> = Default::default();
        before_insert(&mut xx.item);
        self.items.insert(self.last_id, xx);
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

    pub fn item_rect_err(&self, item_id: Id) -> Result<&ItemWithCalcSize<ND>, ItemNotFound> {
        match self.items.get(&item_id) {
            Some(x) => Ok(x),
            None => Err(ItemNotFound(item_id)),
        }
    }

    pub fn item_rect_mut_err(
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

/// Chop a line into pixel-perfect pieces
/// chop(X, 0) => []
/// chop(X, X) => [X]
/// chop(9, 2) => [5, 4]
/// chop(18, 4) => [5, 4, 5, 4]
fn enlarge_gaps_inbetween(gaps_before: &mut Vec<(Id, i16)>, extra_space: i16) {
    let n = gaps_before.len() as Scalar;
    let n = if n >= 2 { n - 1 } else { return };

    let segment = extra_space / n;
    let extra_space = extra_space - segment * n;

    for i in 1..n {
        gaps_before[i as usize].1 += if i <= extra_space {
            segment + 1
        } else {
            segment
        };
    }
}
