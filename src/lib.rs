mod item;
pub mod error;

use error::ItemNotFound;
use item::*;

pub type Scalar = i16;

type Vec2 = [Scalar; 2];
type Vec4 = [Scalar; 4];


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
    
    // ======    
    // run layout algorithm
    // vvvvvv
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

    // ======    
    // adding item
    // vvvvvv
     
    /// add new item with no parent and no children
    pub fn item_new(&mut self) -> &Id {
        self.last_id.0 += 1;
        self.items.insert(self.last_id, (Item::default(), [0, 0, 0, 0]));
        &self.last_id
    }

    // ======    
    // accessing
    // vvvvvv
 
    pub fn items_count(&self) -> usize {
        self.items.len()
    }
       
    pub fn item(&self, item_id: Id) -> Option<&Item> {
        self.items.get(&item_id).map(|x| &(*x).0)
    }

    pub fn item_mut(&mut self, item_id: Id) -> Option<&mut Item> {
        self.items.get_mut(&item_id).map(|x| &mut (*x).0)
    }

    /// Error version of `self.item`
    pub fn item_err(&self, item_id: Id) -> Result<&Item, ItemNotFound> {
        match self.item(item_id) {
            Some(x) => Ok(x),
            None => ItemNotFound(item_id),
        }
    }

    /// Error version of `self.item_mut`
    pub fn item_mut_err(&mut self, item_id: Id) -> Result<&mut Item, ItemNotFound> {
        match self.item_mut(item_id) {
            Some(x) => Ok(x),
            None => ItemNotFound(item_id),
        }
    }

    pub fn item_last_child(&self, parent_id: Id) -> Result<Option<Id>, ItemNotFound> {
        let parent = self.item_err(parent_id)?;
        match parent.first_child {
            None => Ok(None),
            Some(mut r) => {
                'find_last: loop {
                    let r_item = self.item_err(r)?;
                    match r_item.next_sibling {
                        Some(r_next) => r = r_next,
                        None => break 'find_last,
                    }
                }
                Ok(Some(r))
            }
        }
    }


    // ======    
    // Changing hiearchy
    // vvvvvv

    /// insert `later` item as next sibling of `earlier` item
    pub fn insert_after(&mut self, earlier_id: Id, later_id: Id) -> Result<(), ItemNotFound> {
        let earlier = self.item_mut_err(earlier_id)?;
        let later = self.item_mut_err(later_id)?;
        later.next_sibling = earlier.next_sibling;
        later.flags.extra.inserted = true;
        earlier.next_sibling = Some(*later_id);
        Ok(())
    }

    /// set parent of an item, then insert as first child
    pub fn push_child_front(&mut self, parent_id: Id, child_id: Id) {
        unimplemented!()
    }

    /// set parent of an item, then insert as last child
    pub fn push_child_back(&mut self, parent_id: Id, child_id: Id) {
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
