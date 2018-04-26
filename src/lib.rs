#[cfg(test)]
#[macro_use]
extern crate proptest;

extern crate libc;

pub mod errors;
pub mod constants;
mod capabilities;
mod node;
mod resource;

use self::libc::{c_int, size_t};
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use errors::DoogieError;
use constants::{DelimType, IterEventType, ListType, NodeType};

pub type DoogieResult<T> = Result<T, DoogieError>;
type NodeResource = Rc<CMarkNodeResource>;
type SharedResourceMut<T> = Rc<RefCell<T>>;

pub enum CMarkNodePtr {}
enum CMarkIterPtr {}

#[link(name = "cmark")]
extern "C" {
    fn cmark_parse_document(buffer: *const u8, len: size_t, options: c_int) -> *mut CMarkNodePtr;
}

/// Parses the text of a CommonMark document and returns the root node of the document tree.
///
/// # Examples
///
/// ```
/// use doogie::parse_document;
///
/// let document = "# My Great Document \
/// \
/// * Item 1 \
/// * Item 2 \
/// * Item 3";
///
/// let root = parse_document(document);
/// ```
pub fn parse_document(buffer: &str) -> Node {
    unsafe {
        let buffer = buffer.as_bytes();
        let buffer_len = buffer.len() as size_t;
        let p_buffer = buffer.as_ptr();
        let root_ptr = cmark_parse_document(p_buffer, buffer_len, 0);
        let manager = ResourceManager::make_shared();
        let resource = manager.borrow_mut().resource_for(root_ptr);
        let capabilities = CapabilityFactory::new()
            .with_all()
            .build(&resource, manager.clone());

        Node::new(capabilities)
    }
}

/// Represents a node in the document tree.
pub struct Node {
    /// The set of capabilities this node possesses.
    pub capabilities: NodeCapabilities,
}

impl Node {
    /// Create a new Node instance from a set of capabilities.
    fn new(capabilities: NodeCapabilities) -> Node {
        Node { capabilities }
    }
}

pub struct NodeFactory {
    capability_factory: CapabilityFactory,
}

/// The set of capabilities a Node instance possesses.
///
/// All methods available on a Node instance are invoked through the fields of this object.
pub struct NodeCapabilities {
    pub get: Option<NodeGetter>,
    pub set: Option<NodeSetter>,
    pub traverse: Option<NodeTraverser>,
    pub mutate: Option<StructuralMutator>,
    pub render: Option<NodeRenderer>,
    destruct: Option<NodeDestructor>,
}

/// Provides read-only access to Node fields.
pub struct NodeGetter {
    /// NodeResource of the accessed Node.
    resource: NodeResource,
}

impl NodeGetter {
    /// Construct a new NodeGetter instance.
    fn new(resource: NodeResource) -> NodeGetter {
        NodeGetter { resource }
    }
}

/// Write-only access to Node fields.
pub struct NodeSetter {
    /// NodeResource of the target Node.
    resource: NodeResource,
}

impl NodeSetter {
    /// Construct a new NodeSetter instance.
    fn new(resource: NodeResource) -> NodeSetter {
        NodeSetter { resource }
    }
}

/// Methods to access adjacent Nodes in the document tree.
///
/// The capabilities of Nodes accessed through the NodeTraverser methods will match the capabilities
/// of the Node from which they are accessed.
pub struct NodeTraverser {
    /// NodeResource of the current Node.
    resource: NodeResource,
    /// Nodes accessed through the NodeTraverser methods will have their resources managed through
    /// This ResourceManager.
    manager: SharedResourceMut<ResourceManager>,
    /// Factory for constructing the capabilities for the accessed Nodes.
    cap_factory: CapabilityFactory,
}

impl NodeTraverser {
    /// Construct a new NodeTraverser instance.
    fn new(
        resource: NodeResource,
        manager: SharedResourceMut<ResourceManager>,
        cap_factory: CapabilityFactory,
    ) -> NodeTraverser {
        NodeTraverser {
            resource,
            manager,
            cap_factory,
        }
    }
}

/// Contains methods for mutating the structure of a document tree.
pub struct StructuralMutator {
    /// NodeResource of the current Node.
    resource: NodeResource,
    /// ResourceManager that is tracking the current Node.
    manager: SharedResourceMut<ResourceManager>,
    /// Capability factory that will be inherited by any mutated document sub-trees.
    cap_factory: CapabilityFactory,
}

impl StructuralMutator {
    /// Construct a new StructuralMutator instance.
    fn new(
        resource: NodeResource,
        manager: SharedResourceMut<ResourceManager>,
        cap_factory: CapabilityFactory,
    ) -> StructuralMutator {
        StructuralMutator {
            resource,
            manager,
            cap_factory,
        }
    }
}

/// Methods for rendering a document tree into supported textual formats.
pub struct NodeRenderer {
    /// The NodeResource of the current Node.
    resource: NodeResource,
}

impl NodeRenderer {
    /// Construct a new NodeRenderer instance.
    fn new(resource: NodeResource) -> NodeRenderer {
        NodeRenderer { resource }
    }
}

/// Iterator over the subtree rooted in the current node.
///
/// NodeIterator is a wrapper around the libcmark iterator and so traverses the subtree using the
/// same scheme documented [here](https://github.com/commonmark/cmark/blob/a5c83d7a426bda38aac838f9815664f6189d3404/src/cmark.h#L151).
///
/// # Examples
///
/// ```
/// use doogie::parse_document;
/// use doogie::constants::NodeType;
///
/// let doc = "# My awesome document";
/// let node = parse_document(doc);
///
/// if let Some(ref traverser) = node.capabilities.traverse {
///     for (node, event) in traverser.iter() {
///         // Do Some stuff
///     }
/// }
/// ```
pub struct NodeIterator {
    /// Raw CMark iterator pointer.
    iter_p: Option<*mut CMarkIterPtr>,
    /// ResourceManager that will manage the resources of any returned Nodes.
    manager: SharedResourceMut<ResourceManager>,
    /// Iterated Nodes will receive their capabilities from this factory.
    capability_factory: CapabilityFactory,
}

impl NodeIterator {
    /// Construct a new iterator instance.
    fn new(
        iter_p: Option<*mut CMarkIterPtr>,
        manager: SharedResourceMut<ResourceManager>,
        capability_factory: CapabilityFactory,
    ) -> NodeIterator {
        NodeIterator {
            iter_p,
            manager,
            capability_factory,
        }
    }
}

/// Frees the resources of the current Node.
///
/// Only the root nodes should receive this capability since the underlying cmark implementation
/// recursively frees the memory of all descendant nodes as well.
struct NodeDestructor {
    /// NodeResource for the current Node.
    resource: NodeResource,
    /// Manager for the resources of this Node and its descendants.
    manager: Rc<RefCell<ResourceManager>>,
}

/// Encapsulates the memory resources of a Node as allocated by libcmark.
pub struct CMarkNodeResource {
    /// Unique id
    pub id: u32,
    /// Raw Node pointer.
    node_pointer: RefCell<Option<*mut CMarkNodePtr>>,
}

/// Manages the memory resources of Node instances.
///
/// This object is essentially an intermediary between Nodes that are instantiated through this
/// crate and the underlying memory as allocated by libcmark. It provides a mechanism to handle
/// circumstances such as a root node being freed, but an instance of one of its descendants still
/// being in scope somewhere. libcmark recursively frees memory so even though there is still a Node
/// instance, it's underlying memory will have been deallocated. In this event, the NodeResource
/// will be retained by the manager, but the Option containing the pointer will be cleared.
/// Subsequent calls to any of the Node's capabilities will return a ResourceUnavailable error.
pub struct ResourceManager {
    /// Data structure that tracks the resources.
    resources: HashMap<u32, NodeResource>,
}

/// Configurable factory for constructing NodeCapability objects.
pub struct CapabilityFactory {
    /// Constructs a NodeGetter if present.
    getter_builder: Option<Box<Fn(NodeResource) -> NodeGetter>>,
    /// Constructs a NodeSetter if present.
    setter_builder: Option<Box<Fn(NodeResource) -> NodeSetter>>,
    /// Constructs a NodeDestructor if present.
    destructor_builder:
        Option<Box<Fn(NodeResource, SharedResourceMut<ResourceManager>) -> NodeDestructor>>,
    /// Constructs a NodeTraverser if present.
    traverser_builder: Option<
        Box<
            Fn(NodeResource, SharedResourceMut<ResourceManager>, CapabilityFactory)
                -> NodeTraverser,
        >,
    >,
    /// Constructs a StructuralMutator if present.
    mutator_builder: Option<
        Box<
            Fn(NodeResource, SharedResourceMut<ResourceManager>, CapabilityFactory)
                -> StructuralMutator,
        >,
    >,
    /// Constructs a NodeRenderer if present.
    renderer_builder: Option<Box<Fn(NodeResource) -> NodeRenderer>>,
}

#[cfg(test)]
mod tests {
    use super::{parse_document, IterEventType, Node, NodeType, ResourceManager, SharedResourceMut};

    #[test]
    fn test_new_node_gets_getter() {
        let body = "\
        # My Doogie Document
        ";

        let node = parse_document(body);

        assert!(node.capabilities.get.is_some());
    }

    #[test]
    fn test_new_node_gets_setter() {
        let body = "\
        # My Doogie Document
        ";

        let node = parse_document(body);

        assert!(node.capabilities.set.is_some());
    }

    #[test]
    fn test_new_node_gets_traverser() {
        let body = "\
        # My Doogie Document
        ";

        let node = parse_document(body);

        assert!(node.capabilities.traverse.is_some());
    }

    #[test]
    fn test_new_node_gets_mutator() {
        let body = "\
        # My Doogie Document
        ";

        let node = parse_document(body);

        assert!(node.capabilities.mutate.is_some());
    }

    #[test]
    fn test_new_node_gets_renderer() {
        let body = "\
        # My Doogie Document
        ";

        let node = parse_document(body);

        assert!(node.capabilities.render.is_some());
    }

    #[test]
    fn test_new_node_gets_destructor() {
        let body = "\
        # My Doogie Document
        ";

        let node = parse_document(body);

        assert!(node.capabilities.destruct.is_some());
    }

    #[test]
    fn test_iterator_items_have_getter() {
        let body = "\
        * Item 1
        * Item 2
        * Item 3
        ";
        let node = parse_document(body);
        let mut result = false;

        if let Some(ref traverser) = node.capabilities.traverse {
            if let Some((_node, _)) = traverser.iter().next() {
                result = _node.capabilities.get.is_some();
            }
        }

        assert!(result);
    }

    #[test]
    fn test_iterator_items_have_setter() {
        let body = "\
        * Item 1
        * Item 2
        * Item 3
        ";
        let node = parse_document(body);
        let mut result = false;

        if let Some(ref traverser) = node.capabilities.traverse {
            if let Some((_node, _)) = traverser.iter().next() {
                result = _node.capabilities.set.is_some();
            }
        }

        assert!(result);
    }

    #[test]
    fn test_iterator_items_have_traverser() {
        let body = "\
        * Item 1
        * Item 2
        * Item 3
        ";
        let node = parse_document(body);
        let mut result = false;

        if let Some(ref traverser) = node.capabilities.traverse {
            if let Some((_node, _)) = traverser.iter().next() {
                result = _node.capabilities.traverse.is_some();
            }
        }

        assert!(result);
    }

    #[test]
    fn test_iterator_items_have_mutator() {
        let body = "\
        * Item 1
        * Item 2
        * Item 3
        ";
        let node = parse_document(body);
        let mut result = false;

        if let Some(ref traverser) = node.capabilities.traverse {
            if let Some((_node, _)) = traverser.iter().next() {
                result = _node.capabilities.mutate.is_some();
            }
        }

        assert!(result);
    }

    #[test]
    fn test_iterator_items_have_renderer() {
        let body = "\
        * Item 1
        * Item 2
        * Item 3
        ";
        let node = parse_document(body);
        let mut result = false;

        if let Some(ref traverser) = node.capabilities.traverse {
            if let Some((_node, _)) = traverser.iter().next() {
                result = _node.capabilities.render.is_some();
            }
        }

        assert!(result);
    }

    #[test]
    fn test_iterator_items_omit_destructor() {
        let body = "\
        * Item 1
        * Item 2
        * Item 3
        ";
        let node = parse_document(body);
        let mut result = false;

        if let Some(ref traverser) = node.capabilities.traverse {
            if let Some((_node, _)) = traverser.iter().next() {
                result = _node.capabilities.destruct.is_none();
            }
        }

        assert!(result);
    }

    #[test]
    fn test_iterator_items_have_valid_resources() {
        let body = "\
        * Item 1
        * Item 2
        * Item 3
        ";
        let node = parse_document(body);

        if let Some(ref traverser) = node.capabilities.traverse {
            for (node, _event) in traverser.iter() {
                if let Some(ref getter) = node.capabilities.get {
                    if let Err(_) = getter.get_type() {
                        assert!(false);
                    }
                } else {
                    assert!(false);
                }
            }
        }
    }

    #[test]
    fn test_iterator_hits_all_items() {
        let body = "* Item 1\n* Item 2\n* Item 3";
        let node = parse_document(body);
        let mut node_content: Vec<String> = Vec::new();
        let mut item_count = 0;

        if let Some(ref traverser) = node.capabilities.traverse {
            for (node, event) in traverser.iter() {
                if let Some(ref getter) = node.capabilities.get {
                    if let Ok(t) = getter.get_type() {
                        if t == NodeType::CMarkNodeItem && event == IterEventType::Enter {
                            item_count += 1;
                        }
                    }

                    if let Ok(content) = getter.get_content() {
                        node_content.push(String::from(content.trim()));
                    }
                }
            }
        }

        assert_eq!(item_count, 3);
        assert!(node_content.contains(&String::from("Item 1")));
        assert!(node_content.contains(&String::from("Item 2")));
        assert!(node_content.contains(&String::from("Item 3")));
    }

    #[test]
    fn test_iterated_nodes_are_tracked() {
        let body = "* Item 1\n* Item 2\n* Item 3";
        let node = parse_document(body);
        let mut node_count = 0;

        if let Some(ref cap) = node.capabilities.traverse {
            for (_node, event) in cap.iter() {
                if event == IterEventType::Enter {
                    node_count += 1;
                }
            }

            assert_eq!(cap.manager.borrow().resources.len(), node_count);
        } else {
            panic!("Test node should have traverser");
        }
    }

    #[test]
    fn test_iterated_node_tracking_not_duplicated() {
        let body = "* Item 1\n* Item 2\n* Item 3";
        let node = parse_document(body);

        if let Some(ref traverser) = node.capabilities.traverse {
            let mut node_count = 0;

            for (_node, event) in traverser.iter() {
                if event == IterEventType::Enter {
                    node_count += 1;
                }
            }

            for (_node, event) in traverser.iter() {
                if event == IterEventType::Enter {
                    node_count += 1;
                }
            }

            assert_eq!(traverser.manager.borrow().resources.len() * 2, node_count);
        } else {
            panic!("Test node should have traverser");
        }
    }

    #[test]
    fn test_iterated_node_resources_get_invalidated() {
        let body = "* Item 1\n* Item 2\n* Item 3";
        let manager: SharedResourceMut<ResourceManager>;
        let mut nodes: Vec<Node> = Vec::new();

        {
            let node = parse_document(body);
            if let Some(ref traverser) = node.capabilities.traverse {
                manager = traverser.manager.clone();
                for (node, event) in traverser.iter() {
                    if event == IterEventType::Enter {
                        nodes.push(node);
                    }
                }
            } else {
                panic!("Test node should have traverser");
            }
        }

        for (_id, resource) in manager.borrow().resources.iter() {
            assert!(!resource.is_valid());
        }

        for node in &nodes {
            if let Some(ref getter) = node.capabilities.get {
                assert!(!getter.resource.is_valid());
            }
        }

        for node in &nodes {
            if let Some(ref getter) = node.capabilities.get {
                let result = getter.get_type();
                println!("{:?}", result);
                assert!(result.is_err());
            }
        }
    }
}
