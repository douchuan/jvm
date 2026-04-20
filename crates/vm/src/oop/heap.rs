use crate::oop::RefKindDesc;
use std::sync::{Arc, RwLock};

/// Slot-based heap for JVM objects.
///
/// Objects are allocated to slots identified by `u32`. `Oop::Ref(slot_id)`
/// holds the ID, and the actual data lives here. This design enables:
/// - Zero unsafe code for object access
/// - GC can move objects by updating internal slot mapping
/// - `Arc<RwLock<T>>` provides safe concurrent access
pub struct Heap {
    slots: Vec<Option<Arc<RwLock<RefKindDesc>>>>,
    free_list: Vec<u32>,
}

impl Heap {
    pub fn new() -> Self {
        Self {
            slots: Vec::new(),
            free_list: Vec::new(),
        }
    }

    /// Allocate a new object, return its slot ID.
    pub fn alloc(&mut self, desc: RefKindDesc) -> u32 {
        if let Some(id) = self.free_list.pop() {
            self.slots[id as usize] = Some(Arc::new(RwLock::new(desc)));
            id
        } else {
            let id = self.slots.len() as u32;
            self.slots.push(Some(Arc::new(RwLock::new(desc))));
            id
        }
    }

    /// Get the Arc handle for a slot. The caller can then `.read()` or `.write()`
    /// to access the data safely.
    pub fn get(&self, slot_id: u32) -> Arc<RwLock<RefKindDesc>> {
        self.slots[slot_id as usize].clone().expect("slot is alive")
    }

    /// Free a slot, making its slot_id available for reuse.
    pub fn free(&mut self, slot_id: u32) {
        self.slots[slot_id as usize] = None;
        self.free_list.push(slot_id);
    }

    /// Check if two slot IDs refer to the same object (pointer equality).
    pub fn is_same_slot(&self, a: u32, b: u32) -> bool {
        match (self.slots.get(a as usize), self.slots.get(b as usize)) {
            (Some(Some(a_arc)), Some(Some(b_arc))) => Arc::ptr_eq(a_arc, b_arc),
            _ => false,
        }
    }
}
