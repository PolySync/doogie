extern crate libc;
#[cfg(test)]
extern crate proptest;
extern crate try_from;

use std::fmt::{Debug, Formatter};
use std::fmt;
use self::try_from::TryFrom;
use self::libc::{c_int, c_void};
use std::rc::Rc;
use std::cell::RefCell;
use super::{CMarkIterPtr, CMarkNodePtr, CapabilityFactory, IterEventType, Node, NodeDestructor,
            NodeFactory, NodeIterator, NodeResource, NodeType, ResourceManager};

#[link(name = "cmark")]
extern "C" {
    fn cmark_node_free(node: *mut CMarkNodePtr);

    fn cmark_iter_free(iter: *mut CMarkIterPtr) -> c_void;

    fn cmark_iter_next(iter: *mut CMarkIterPtr) -> c_int;

    fn cmark_iter_get_node(iter: *mut CMarkIterPtr) -> *mut CMarkNodePtr;

    fn cmark_node_new(node_type: u32) -> *mut CMarkNodePtr;
}

impl NodeFactory {
    pub fn new(capability_factory: CapabilityFactory) -> Self {
        NodeFactory { capability_factory }
    }

    pub fn build(&self, node_type: NodeType) -> Node {
        unsafe {
            let node_ptr = cmark_node_new(node_type as u32);
            let manager = ResourceManager::make_shared();
            let resource = manager.borrow_mut().resource_for(node_ptr);
            Node::new(self.capability_factory.build(&resource, manager))
        }
    }
}

impl Drop for Node {
    /// Free the NodeResource
    fn drop(&mut self) {
        if let Some(ref destructor) = self.capabilities.destruct {
            destructor.free();
        }
    }
}

impl Debug for Node {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut type_string = "Unavailable".to_string();
        if let Some(getter) = self.capabilities.get.as_ref() {
            if let Ok(node_type) = getter.get_type_string() {
                type_string = node_type;
            }
        }

        write!(f, "<{} Node>", type_string)
    }
}

impl NodeDestructor {
    /// Construct a new NodeDestructor instance.
    pub fn new(resource: NodeResource, manager: Rc<RefCell<ResourceManager>>) -> NodeDestructor {
        NodeDestructor {
            resource: resource,
            manager: manager,
        }
    }

    /// Free the resources associated with this NodeDestructor.
    pub fn free(&self) {
        let node_p: Option<*mut CMarkNodePtr>;
        {
            node_p = *self.resource.node_pointer.borrow_mut();
        }

        let mut _manager = self.manager.borrow_mut();
        _manager.invalidate_resource(&self.resource);

        if let Some(p) = node_p {
            unsafe {
                cmark_node_free(p);
            }
        }
    }
}

impl Iterator for NodeIterator {
    type Item = (Node, IterEventType);

    /// Advance the iterator.
    ///
    /// # Examples
    ///
    /// ```
    /// use doogie::parse_document;
    ///
    /// let doc = "# My awesome document";
    /// let node = parse_document(doc);
    ///
    /// if let Some(ref traverser) = node.capabilities.traverse {
    ///     let mut iter = traverser.iter();
    ///     while let Some((_node, _event)) = iter.next() {
    ///         // Do something
    ///     }
    /// }
    /// ```
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(p) = self.iter_p {
            unsafe {
                match IterEventType::try_from(cmark_iter_next(p) as u32) {
                    Ok(IterEventType::Done) | Ok(IterEventType::None) => None,
                    Ok(event) => {
                        let node_p = cmark_iter_get_node(p);
                        let mut manager = self.manager.borrow_mut();
                        let resource = manager.resource_for(node_p);
                        let capabilities = self.capability_factory
                            .build(&resource, self.manager.clone());
                        let node = Node::new(capabilities);
                        Some((node, event))
                    }
                    _ => None,
                }
            }
        } else {
            return None;
        }
    }
}

impl Drop for NodeIterator {
    /// Free the CMark memory allocated for the iterator.
    fn drop(&mut self) {
        if let Some(p) = self.iter_p {
            unsafe {
                cmark_iter_free(p);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::proptest::prelude::*;
    use {parse_document, CapabilityFactory, DoogieResult, Node, NodeType, ResourceManager};

    fn arb_content(max_words: usize) -> BoxedStrategy<String> {
        prop::collection::vec("[[:alnum:]]{1,45}", 1..max_words)
            .prop_map(|v| v.join(" "))
            .boxed()
    }

    #[test]
    fn parse_and_render() {
        let content = "# Testing";
        let root = parse_document(content);

        if let Some(ref renderer) = root.capabilities.render {
            assert_eq!(content, renderer.render_commonmark().trim());
        } else {
            assert!(false, "No Renderer");
        }
    }

    #[test]
    fn node_type() {
        unsafe {
            let manager = ResourceManager::make_shared();
            let node_pointer = super::cmark_node_new(NodeType::CMarkNodeParagraph as u32);
            let resource = manager.borrow_mut().resource_for(node_pointer);
            let capabilities = CapabilityFactory::new()
                .with_destructor()
                .with_getter()
                .build(&resource, manager);
            let node = Node::new(capabilities);
            let mut result = NodeType::CMarkNodeNone;

            if let Some(ref getter) = node.capabilities.get {
                result = getter.get_type().unwrap();
            }

            assert_eq!(result, NodeType::CMarkNodeParagraph)
        }
    }

    proptest! {
        #[test]
        fn set_and_get_content(ref content in arb_content(10)) {
            unsafe {
                let mut result = String::new();
                let node_pointer = super::cmark_node_new(NodeType::CMarkNodeText as u32);
                let manager = ResourceManager::make_shared();
                let resource = manager.borrow_mut().resource_for(node_pointer);
                let capabilities = CapabilityFactory::new()
                    .with_destructor()
                    .with_getter()
                    .with_setter()
                    .build(&resource, manager);
                let node = Node::new(capabilities);

                if let Some(ref setter) = node.capabilities.set {
                    setter.set_content(content).unwrap();
                }
                if let Some(ref getter) = node.capabilities.get {
                    result = getter.get_content().unwrap();
                }

                assert_eq!(content, &result);
            }
        }
    }

    proptest! {
        #[test]
        fn test_fence_info_get_set(ref content in arb_content(10)){
            unsafe {
                let node_pointer = super::cmark_node_new(NodeType::CMarkNodeCodeBlock as u32);
                let manager = ResourceManager::make_shared();
                let resource = manager.borrow_mut().resource_for(node_pointer);
                let capabilities = CapabilityFactory::new()
                    .with_destructor()
                    .with_getter()
                    .with_setter()
                    .build(&resource, manager);
                let node = Node::new(capabilities);
                let mut result = String::new();

                if let Some(ref setter) = node.capabilities.set {
                    setter.set_fence_info(content).unwrap();
                }
                if let Some(ref getter) = node.capabilities.get {
                    result = getter.get_fence_info().unwrap();
                }

                assert_eq!(content, &result);
            }
        }
    }

    #[test]
    fn test_managed_node_with_destructor_invalidates_resource() {
        unsafe {
            let node_pointer = super::cmark_node_new(NodeType::CMarkNodeParagraph as u32);
            let manager = ResourceManager::make_shared();
            let resource = manager.borrow_mut().resource_for(node_pointer);

            {
                let capabilities = CapabilityFactory::new()
                    .with_destructor()
                    .build(&resource, manager);
                let _node = Node::new(capabilities);
            }

            assert!(!resource.is_valid());
        }
    }

    #[test]
    fn test_shared_node_resource_is_invalidated_for_all_sharing_nodes() {
        unsafe {
            let node_pointer = super::cmark_node_new(NodeType::CMarkNodeParagraph as u32);
            let manager = ResourceManager::make_shared();
            let resource = manager.borrow_mut().resource_for(node_pointer);
            let mut result: DoogieResult<String> = Ok(String::new());
            let mut cap_factory = CapabilityFactory::new().with_getter();

            let node_one = Node::new(cap_factory.build(&resource, manager.clone()));
            {
                cap_factory = cap_factory.with_destructor();
                Node::new(cap_factory.build(&resource, manager.clone()));
            }
            if let Some(ref getter) = node_one.capabilities.get {
                result = getter.get_content();
            }

            assert!(result.is_err());
        }
    }

    #[test]
    fn test_multiple_nodes_with_destructors_do_not_double_free() {
        unsafe {
            let pointer = super::cmark_node_new(NodeType::CMarkNodeParagraph as u32);
            let manager = ResourceManager::make_shared();
            let resource = manager.borrow_mut().resource_for(pointer);
            let cap_factory = CapabilityFactory::new().with_destructor();
            {
                let _node_one = Node::new(cap_factory.build(&resource, manager.clone()));
                let _node_two = Node::new(cap_factory.build(&resource, manager.clone()));
            }

            assert!(!resource.is_valid());
        }
    }

    #[test]
    fn test_getter_returns_error_when_resource_invalid() {
        unsafe {
            let pointer = super::cmark_node_new(NodeType::CMarkNodeParagraph as u32);
            let manager = ResourceManager::make_shared();
            let resource = manager.borrow_mut().resource_for(pointer);

            let node = Node::new(
                CapabilityFactory::new()
                    .with_getter()
                    .build(&resource, manager.clone()),
            );
            {
                Node::new(
                    CapabilityFactory::new()
                        .with_destructor()
                        .build(&resource, manager.clone()),
                );
            }

            if let Some(ref getter) = node.capabilities.get {
                if let Ok(_) = getter.get_type() {
                    assert!(false);
                }
            } else {
                panic!("Test node should have getter");
            }
        }
    }

    #[test]
    fn test_first_child() {
        let content = "A single paragraph";
        let root = parse_document(content);
        let traverser = root.capabilities.traverse.as_ref().unwrap();
        assert!(traverser.first_child().unwrap().is_some());
        assert!(traverser.first_child().unwrap().is_some());
    }
}
