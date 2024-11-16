
use std::cell::Cell;
use std::marker::PhantomData;

// Base class for objects stored in the array. If you want to store
// same object in multiple arrays, each of those arrays has to have
// different ID. The item itself has to be derived from instantiations of
// array_item_t template for all relevant IDs.


pub trait ArrayItem<ID> {
    // array_index: Cell<isize>,
    // _marker: PhantomData<ID>,
    fn get_array_index() -> Cell<isize>;
    fn set_array_index(&self, index: isize);

    fn get_marker(&self) -> PhantomData<ID>;
    fn set_marker(&mut self, in_phantom_data: &mut PhantomData<ID>);
}

pub struct Array<T, ID> {
    items: Vec<Option<T>>,
    _marker: PhantomData<ID>,
}

impl<T, ID> Array<T, ID>
where
    T: ArrayItem<ID>,
{
    pub fn new() -> Self {
        Array {
            items: Vec::new(),
            _marker: PhantomData,
        }
    }

    pub fn size(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.items.get(index).and_then(|item| item.as_ref())
    }

    pub fn push_back(&mut self, item: T) {
        let index = self.items.len() as isize;
        item.set_array_index(index);
        self.items.push(Some(item));
    }

    pub fn erase(&mut self, item: &T) {
        let index = item.get_array_index() as usize;
        self.erase_at(index);
    }

    pub fn erase_at(&mut self, index: usize) {
        if self.items.is_empty() {
            return;
        }
        if let Some(last_item) = self.items.pop().flatten() {
            last_item.set_array_index(index as isize);
            self.items[index] = Some(last_item);
        }
    }

    pub fn swap(&mut self, index1: usize, index2: usize) {
        if let Some(item1) = self.items.get(index1).and_then(|item| item.as_ref()) {
            item1.set_array_index(index2 as isize);
        }
        if let Some(item2) = self.items.get(index2).and_then(|item| item.as_ref()) {
            item2.set_array_index(index1 as isize);
        }
        self.items.swap(index1, index2);
    }

    pub fn clear(&mut self) {
        self.items.clear();
    }

    pub fn index(item: &T) -> usize {
        item.get_array_index() as usize
    }
}
