pub type Scalar = i16;

type Vec2 = [Scalar; 2];
type Vec4 = [Scalar; 4];

mod item;
use item::*;

#[derive(Debug)]
pub struct Context {
    last_id: Id,
    pub(crate) items: std::collections::BTreeMap<Id, (Item, Vec4)>,
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

impl Context {
    pub fn new() -> Self {
        let mut items = Default::default();
        Self {
            last_id: 0,
            items,
        }
    }

    pub fn run() {
        unimplemented!()
    }
    pub fn run_item(&mut self, item_id: Id) {
        self.calc_size(item_id, 0);
        self.calc_size(item_id, 1);
        self.arrange(item_id, 0);
        self.arrange(item_id, 1);
    }

    fn calc_size(&mut self, item_id: Id, dim: usize) {
        unimplemented!()
    }

    fn arrange(&mut self, item_id: Id, dim: usize) {
        unimplemented!()
    }

    pub fn item(&mut self, item_id: Id) -> Option<&mut Item> {
        self.items.get_mut(&item).map(|x| &mut (*x).0)
    }

    pub fn items_count(&self) -> usize {
        self.items.len()
    }

    /// add new item with no parent
    pub fn add_item(&mut self) -> &Id {
        self.last_id.0 += 1;
        self.items.insert(self.last_id, (Item::default(), [0, 0, 0, 0]));
        &self.last_id
    }

    /// set parent of an item, then insert as first child
    pub fn push_child_front(&mut self, parent: &Id, child: &Id) {
        unimplemented!()
    }

    /// set parent of an item, then insert as last child
    pub fn push_child_back(&mut self, parent: &Id, child: &Id) {
        unimplemented!()
    }

    /// insert `later` item as next sibling of `earlier` item
    pub fn insert_after(&mut self, earlier: &Id, later: &Id) {
        unimplemented!()
    }
}

#[derive(Default, Clone, Debug)]
pub enum Alignment {
    #[default]
    Start,
    Center,
    End,
    Justify,
}

fn main() {
    println!("Hello, world!");
}
