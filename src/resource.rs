extern crate libc;
extern crate try_from;

use self::libc::c_int;
use self::try_from::TryFrom;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use super::{CMarkIterPtr, CMarkNodePtr, CMarkNodeResource, IterEventType, NodeResource,
            ResourceManager, SharedResourceMut};

extern "C" {
    fn cmark_iter_new(node: *mut CMarkNodePtr) -> *mut CMarkIterPtr;

    fn cmark_iter_next(iter: *mut CMarkIterPtr) -> c_int;

    fn cmark_iter_get_node(iter: *mut CMarkIterPtr) -> *mut CMarkNodePtr;
}

impl CMarkNodeResource {
    /// Construct a new CMarkNodeResource instance.
    pub fn new(pointer: *mut CMarkNodePtr) -> CMarkNodeResource {
        let node_pointer = RefCell::new(Some(pointer));

        CMarkNodeResource {
            id: pointer as u32,
            node_pointer,
        }
    }

    #[cfg(test)]
    pub fn is_valid(&self) -> bool {
        self.node_pointer.borrow().is_some()
    }
}

impl ResourceManager {
    /// Construct a new ResourceManager instance.
    pub fn new() -> ResourceManager {
        ResourceManager {
            resources: HashMap::new(),
        }
    }

    /// Construct a ResourceManager instance managed behind a mutable smart pointer combination.
    pub fn make_shared() -> SharedResourceMut<Self> {
        Rc::new(RefCell::new(ResourceManager::new()))
    }

    /// Get a NodeResource corresponding to a raw libcmark node pointer.
    ///
    /// If the given pointer is not being tracked by this ResourceManager, a tracked NodeResource
    /// will be constructed and returned.
    pub fn resource_for(&mut self, node_pointer: *mut CMarkNodePtr) -> NodeResource {
        let id = node_pointer as u32;

        let resource = self.resources
            .entry(id)
            .or_insert_with(|| Rc::new(CMarkNodeResource::new(node_pointer)));

        resource.clone()
    }

    /// Track a NodeResource with this ResourceManager.
    pub fn track_resource(&mut self, resource: &NodeResource) {
        self.resources.insert(resource.id, resource.clone());
    }

    /// Transfer all tracked NodeResource objects to this ResourceManager.
    pub fn absorb(&mut self, other: &SharedResourceMut<ResourceManager>) {
        let mut _other = other.borrow_mut();
        for (key, val) in _other.resources.drain() {
            self.resources.insert(key, val);
        }
    }

    /// Invalidate the libcmark pointer for the given NodeResource.
    ///
    /// All tracked NodeResource objects that represent descendant Nodes of the given NodeResource
    /// will also be invalidated.
    pub fn invalidate_resource(&mut self, resource: &NodeResource) {
        if let Some(p) = *resource.node_pointer.borrow() {
            unsafe {
                let iter_p = cmark_iter_new(p);

                loop {
                    if let Ok(event) = IterEventType::try_from(cmark_iter_next(iter_p) as u32) {
                        if event == IterEventType::Done {
                            break;
                        }

                        let current_p = cmark_iter_get_node(iter_p);
                        if let Some(resource) = self.resources.get(&(current_p as u32)) {
                            if let Ok(mut pointer) = resource.node_pointer.try_borrow_mut() {
                                pointer.take();
                            }
                        }
                    }
                }
            }
        }

        resource.node_pointer.borrow_mut().take();
    }

    /// Removes the given NodeResource and returns it in a new ResourceManager.
    ///
    /// Any tracked NodeResource objects that represent descendant Nodes of the given NodeResource
    /// will also be removed and added to the returned ResourceManager.
    pub fn prune(&mut self, resource: &NodeResource) -> SharedResourceMut<ResourceManager> {
        let mut manager = ResourceManager::new();

        if let Some(p) = *resource.node_pointer.borrow() {
            unsafe {
                let iter_p = cmark_iter_new(p);

                loop {
                    if let Ok(event) = IterEventType::try_from(cmark_iter_next(iter_p) as u32) {
                        if event == IterEventType::Done {
                            break;
                        }
                        let node_ptr = cmark_iter_get_node(iter_p);
                        let id = node_ptr as u32;
                        if let Some(resource) = self.resources.remove(&id) {
                            manager.track_resource(&resource);
                        }
                    }
                }
            }
        }

        Rc::new(RefCell::new(manager))
    }
}
