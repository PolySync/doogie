extern crate libc;
extern crate try_from;

use self::try_from::TryFrom;
use std::ffi::{CStr, CString};
use self::libc::{c_char, c_int, c_void};
use super::{CMarkIterPtr, CMarkNodePtr, CapabilityFactory, DelimType, DoogieError, DoogieResult,
            ListType, Node, NodeCapabilities, NodeDestructor, NodeGetter, NodeIterator,
            NodeRenderer, NodeResource, NodeSetter, NodeTraverser, NodeType, ResourceManager,
            SharedResourceMut, StructuralMutator};

#[link(name = "cmark")]
extern "C" {
    fn cmark_node_get_literal(node: *mut CMarkNodePtr) -> *const c_char;

    fn cmark_node_set_literal(node: *mut CMarkNodePtr, content: *const c_char) -> c_int;

    fn cmark_node_get_type(node: *mut CMarkNodePtr) -> c_int;

    fn cmark_node_get_type_string(node: *mut CMarkNodePtr) -> *const c_char;

    fn cmark_node_get_start_line(node: *mut CMarkNodePtr) -> c_int;

    fn cmark_node_get_start_column(node: *mut CMarkNodePtr) -> c_int;

    fn cmark_node_get_list_type(node: *mut CMarkNodePtr) -> c_int;

    fn cmark_node_get_list_delim(node: *mut CMarkNodePtr) -> c_int;

    fn cmark_node_get_heading_level(node: *mut CMarkNodePtr) -> c_int;

    fn cmark_node_get_url(node: *mut CMarkNodePtr) -> *const c_char;

    fn cmark_node_get_title(node: *mut CMarkNodePtr) -> *const c_char;

    fn cmark_node_get_fence_info(node: *mut CMarkNodePtr) -> *const c_char;

    fn cmark_node_set_fence_info(node: *mut CMarkNodePtr, info: *const c_char) -> c_int;

    fn cmark_node_next(node: *mut CMarkNodePtr) -> *mut CMarkNodePtr;

    fn cmark_node_previous(node: *mut CMarkNodePtr) -> *mut CMarkNodePtr;

    fn cmark_node_parent(node: *mut CMarkNodePtr) -> *mut CMarkNodePtr;

    fn cmark_node_first_child(node: *mut CMarkNodePtr) -> *mut CMarkNodePtr;

    fn cmark_node_last_child(node: *mut CMarkNodePtr) -> *mut CMarkNodePtr;

    fn cmark_node_unlink(node: *mut CMarkNodePtr) -> c_void;

    fn cmark_node_append_child(node: *mut CMarkNodePtr, child: *mut CMarkNodePtr) -> c_int;

    fn cmark_consolidate_text_nodes(root: *mut CMarkNodePtr) -> c_void;

    fn cmark_render_xml(root: *mut CMarkNodePtr, options: c_int) -> *const c_char;

    fn cmark_render_commonmark(root: *mut CMarkNodePtr, options: c_int) -> *const c_char;

    fn cmark_iter_new(node: *mut CMarkNodePtr) -> *mut CMarkIterPtr;
}

impl CapabilityFactory {
    /// Construct a new CapabilityFactory instance.
    pub fn new() -> Self {
        CapabilityFactory {
            getter_builder: None,
            setter_builder: None,
            destructor_builder: None,
            traverser_builder: None,
            mutator_builder: None,
            renderer_builder: None,
        }
    }

    /// Make a new CapabilityFactory instance that is configured with the same capability set as
    /// this instance except for destructor which should not be replicated for child nodes.
    pub fn make_child_factory(&self) -> Self {
        let mut factory = CapabilityFactory::new();

        if self.getter_builder.is_some() {
            factory = factory.with_getter();
        }
        if self.setter_builder.is_some() {
            factory = factory.with_setter();
        }
        if self.traverser_builder.is_some() {
            factory = factory.with_traverser();
        }
        if self.mutator_builder.is_some() {
            factory = factory.with_mutator();
        }
        if self.renderer_builder.is_some() {
            factory = factory.with_renderer();
        }

        factory
    }

    pub fn with_all(self) -> Self {
        self.with_destructor()
            .with_renderer()
            .with_getter()
            .with_setter()
            .with_traverser()
            .with_mutator()
    }

    /// Configure this factory to produce a getter capability.
    pub fn with_getter(mut self) -> Self {
        self.getter_builder = Some(Box::new(|resource| NodeGetter::new(resource)));

        self
    }

    /// Configure this factory to produce a setter capability.
    pub fn with_setter(mut self) -> Self {
        self.setter_builder = Some(Box::new(|resource| NodeSetter::new(resource)));

        self
    }

    /// Configure this factory to produce a destructor capability.
    pub fn with_destructor(mut self) -> Self {
        self.destructor_builder = Some(Box::new(
            move |resource: NodeResource, manager: SharedResourceMut<ResourceManager>| {
                NodeDestructor::new(resource.clone(), manager.clone())
            },
        ));

        self
    }

    /// Configure this factory to produce a traverser capability.
    pub fn with_traverser(mut self) -> Self {
        self.traverser_builder = Some(Box::new(
            move |resource: NodeResource,
                  manager: SharedResourceMut<ResourceManager>,
                  cap_factory: CapabilityFactory| {
                NodeTraverser::new(resource.clone(), manager.clone(), cap_factory)
            },
        ));

        self
    }

    /// Configure this factory to produce a renderer capability.
    pub fn with_renderer(mut self) -> Self {
        self.renderer_builder = Some(Box::new(|resource: NodeResource| {
            NodeRenderer::new(resource.clone())
        }));

        self
    }

    /// Configure this factory to produce a mutator capability.
    pub fn with_mutator(mut self) -> Self {
        self.mutator_builder = Some(Box::new(
            move |resource: NodeResource,
                  manager: SharedResourceMut<ResourceManager>,
                  cap_factory: CapabilityFactory| {
                StructuralMutator::new(resource.clone(), manager.clone(), cap_factory)
            },
        ));

        self
    }

    /// Build a NodeCapability instance with the configured capabilities for the given NodeResource
    /// and ResourceManager.
    pub fn build(
        &self,
        resource: &NodeResource,
        manager: SharedResourceMut<ResourceManager>,
    ) -> NodeCapabilities {
        NodeCapabilities {
            get: if let Some(ref builder) = self.getter_builder {
                Some((builder)(resource.clone()))
            } else {
                None
            },
            set: if let Some(ref builder) = self.setter_builder {
                Some((builder)(resource.clone()))
            } else {
                None
            },
            traverse: if let Some(ref builder) = self.traverser_builder {
                Some((builder)(
                    resource.clone(),
                    manager.clone(),
                    self.make_child_factory(),
                ))
            } else {
                None
            },
            destruct: if let Some(ref builder) = self.destructor_builder {
                Some((builder)(resource.clone(), manager.clone()))
            } else {
                None
            },
            mutate: if let Some(ref builder) = self.mutator_builder {
                Some((builder)(
                    resource.clone(),
                    manager.clone(),
                    self.make_child_factory(),
                ))
            } else {
                None
            },
            render: if let Some(ref builder) = self.renderer_builder {
                Some((builder)(resource.clone()))
            } else {
                None
            },
        }
    }
}

impl NodeGetter {
    /// Get the textual content of a Node if supported by the Node Type.
    ///
    /// # Examples
    ///
    /// ```
    /// use doogie::parse_document;
    /// use doogie::constants::NodeType;
    ///
    /// let doc = "# My awesome document\n\n* Item One\n*Item Two\n*Item Three";
    /// let magic_word = String::from("Abracadabra");
    /// let node = parse_document(doc);
    ///
    /// for (node, event) in node.capabilities.traverse.as_ref().unwrap().iter() {
    ///     let getter = node.capabilities.get.as_ref().unwrap();
    ///     let setter = node.capabilities.set.as_ref().unwrap();
    ///
    ///     if let Ok(_) = setter.set_content(&magic_word) {
    ///         assert_eq!(getter.get_content().unwrap(), magic_word);
    ///     }
    /// }
    ///
    pub fn get_content(&self) -> DoogieResult<String> {
        if let Some(p) = *self.resource.node_pointer.borrow() {
            unsafe {
                let result = cmark_node_get_literal(p);
                if result.is_null() {
                    return Ok(String::new());
                } else {
                    return Ok(CStr::from_ptr(result).to_str()?.to_string());
                }
            }
        } else {
            return Err(DoogieError::ResourceUnavailable);
        }
    }

    /// Get the CMark Type for this Node.
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
    /// let node_type = node
    ///     .capabilities
    ///     .get
    ///     .as_ref()
    ///     .map(|getter| { getter.get_type().unwrap() });
    ///
    /// assert_eq!(node_type, Some(NodeType::CMarkNodeDocument));
    pub fn get_type(&self) -> DoogieResult<NodeType> {
        if let Some(p) = *self.resource.node_pointer.borrow() {
            unsafe {
                let t = cmark_node_get_type(p);
                Ok(NodeType::try_from(t as u32)?)
            }
        } else {
            Err(DoogieError::ResourceUnavailable)
        }
    }

    pub fn get_type_string(&self) -> DoogieResult<String> {
        if let Some(p) = *self.resource.node_pointer.borrow() {
            unsafe {
                Ok(CStr::from_ptr(cmark_node_get_type_string(p))
                    .to_str()?
                    .to_string())
            }
        } else {
            Err(DoogieError::ResourceUnavailable)
        }
    }

    /// Get the start line from the original CMark document corresponding to this Node.
    pub fn get_start_line(&self) -> DoogieResult<u32> {
        if let Some(p) = *self.resource.node_pointer.borrow() {
            unsafe { Ok(cmark_node_get_start_line(p) as u32) }
        } else {
            Err(DoogieError::ResourceUnavailable)
        }
    }

    /// Get the start column from the original CMark document corresponding to this Node.
    pub fn get_start_column(&self) -> DoogieResult<u32> {
        if let Some(p) = *self.resource.node_pointer.borrow() {
            unsafe { Ok(cmark_node_get_start_column(p) as u32) }
        } else {
            Err(DoogieError::ResourceUnavailable)
        }
    }

    pub fn get_id(&self) -> DoogieResult<u32> {
        if let Some(p) = *self.resource.node_pointer.borrow() {
            Ok(p as u32)
        } else {
            Err(DoogieError::ResourceUnavailable)
        }
    }

    /// Get the CMark list type of the Node if applicable.
    pub fn get_list_type(&self) -> DoogieResult<ListType> {
        if let Some(p) = *self.resource.node_pointer.borrow() {
            unsafe { ListType::try_from(cmark_node_get_list_type(p) as u32) }
        } else {
            Err(DoogieError::ResourceUnavailable)
        }
    }

    /// Get the CMark delimiter type of the Node if applicable.
    pub fn get_delim_type(&self) -> DoogieResult<DelimType> {
        if let Some(p) = *self.resource.node_pointer.borrow() {
            unsafe { DelimType::try_from(cmark_node_get_list_delim(p) as u32) }
        } else {
            Err(DoogieError::ResourceUnavailable)
        }
    }

    /// Get the heading level of the Node if applicable.
    pub fn get_heading_level(&self) -> DoogieResult<u32> {
        if let Some(p) = *self.resource.node_pointer.borrow() {
            unsafe { Ok(cmark_node_get_heading_level(p) as u32) }
        } else {
            Err(DoogieError::ResourceUnavailable)
        }
    }

    /// Get the url of the Node if applicable.
    pub fn get_url(&self) -> DoogieResult<String> {
        if let Some(p) = *self.resource.node_pointer.borrow() {
            unsafe { Ok(CStr::from_ptr(cmark_node_get_url(p)).to_str()?.to_string()) }
        } else {
            Err(DoogieError::ResourceUnavailable)
        }
    }

    /// Get the link title of the Node if applicable.
    pub fn get_title(&self) -> DoogieResult<String> {
        if let Some(p) = *self.resource.node_pointer.borrow() {
            unsafe {
                Ok(CStr::from_ptr(cmark_node_get_title(p))
                    .to_str()?
                    .to_string())
            }
        } else {
            Err(DoogieError::ResourceUnavailable)
        }
    }

    /// Get the code fence info from the Node if applicable.
    pub fn get_fence_info(&self) -> DoogieResult<String> {
        if let Some(p) = *self.resource.node_pointer.borrow() {
            unsafe {
                Ok(CStr::from_ptr(cmark_node_get_fence_info(p))
                    .to_str()?
                    .to_string())
            }
        } else {
            Err(DoogieError::ResourceUnavailable)
        }
    }
}

impl NodeSetter {
    /// Set the textual content of a Node if supported by the Node Type.
    ///
    /// # Examples
    ///
    /// ```
    /// use doogie::parse_document;
    /// use doogie::constants::NodeType;
    ///
    /// let doc = "# My awesome document\n\n* Item One\n*Item Two\n*Item Three";
    /// let magic_word = String::from("Abracadabra");
    /// let node = parse_document(doc);
    ///
    /// for (node, event) in node.capabilities.traverse.as_ref().unwrap().iter() {
    ///     let getter = node.capabilities.get.as_ref().unwrap();
    ///     let setter = node.capabilities.set.as_ref().unwrap();
    ///
    ///     if let Ok(_) = setter.set_content(&magic_word) {
    ///         assert_eq!(getter.get_content().unwrap(), magic_word);
    ///     }
    /// }
    ///
    pub fn set_content(&self, content: &String) -> DoogieResult<u32> {
        if let Some(p) = *self.resource.node_pointer.borrow() {
            unsafe {
                let content = CString::new(content.as_bytes())?;

                match cmark_node_set_literal(p, content.as_ptr()) {
                    1 => Ok(1 as u32),
                    i => Err(DoogieError::ReturnCode(i as u32)),
                }
            }
        } else {
            Err(DoogieError::ResourceUnavailable)
        }
    }

    /// Set the code fence info of a Node if supported by the Node Type.
    pub fn set_fence_info(&self, info: &String) -> DoogieResult<u32> {
        if let Some(p) = *self.resource.node_pointer.borrow() {
            unsafe {
                let info = CString::new(info.as_bytes())?;

                match cmark_node_set_fence_info(p, info.as_ptr()) {
                    1 => Ok(1),
                    err => Err(DoogieError::ReturnCode(err as u32)),
                }
            }
        } else {
            Err(DoogieError::ResourceUnavailable)
        }
    }
}

impl NodeTraverser {
    /// Get the next node at the same hierarchical level in the document tree if any.
    pub fn next_sibling(&self) -> DoogieResult<Option<Node>> {
        if let Some(p) = *self.resource.node_pointer.borrow() {
            unsafe {
                let next_node_ptr = cmark_node_next(p);

                if next_node_ptr.is_null() {
                    Ok(None)
                } else {
                    let mut manager = self.manager.borrow_mut();
                    let resource = manager.resource_for(next_node_ptr);
                    let capabilities = self.cap_factory.build(&resource, self.manager.clone());
                    Ok(Some(Node::new(capabilities)))
                }
            }
        } else {
            Err(DoogieError::ResourceUnavailable)
        }
    }

    /// Get the previous node at the same hierarchical level in the document tree if any.
    pub fn prev_sibling(&self) -> DoogieResult<Option<Node>> {
        if let Some(p) = *self.resource.node_pointer.borrow() {
            unsafe {
                let prev_node_ptr = cmark_node_previous(p);

                if prev_node_ptr.is_null() {
                    Ok(None)
                } else {
                    let mut manager = self.manager.borrow_mut();
                    let resource = manager.resource_for(prev_node_ptr);
                    let capabilities = self.cap_factory.build(&resource, self.manager.clone());
                    Ok(Some(Node::new(capabilities)))
                }
            }
        } else {
            Err(DoogieError::ResourceUnavailable)
        }
    }

    /// Get the immediate parent of the current node in the document tree if any.
    pub fn parent(&self) -> DoogieResult<Option<Node>> {
        if let Some(p) = *self.resource.node_pointer.borrow() {
            unsafe {
                let parent_node_ptr = cmark_node_parent(p);

                if parent_node_ptr.is_null() {
                    Ok(None)
                } else {
                    let mut manager = self.manager.borrow_mut();
                    let resource = manager.resource_for(parent_node_ptr);
                    let capabilities = self.cap_factory.build(&resource, self.manager.clone());
                    Ok(Some(Node::new(capabilities)))
                }
            }
        } else {
            Err(DoogieError::ResourceUnavailable)
        }
    }

    /// Get the first child Node of the current Node if any.
    pub fn first_child(&self) -> DoogieResult<Option<Node>> {
        if let Some(p) = *self.resource.node_pointer.borrow() {
            unsafe {
                let child_ptr = cmark_node_first_child(p);

                if child_ptr.is_null() {
                    Ok(None)
                } else {
                    let mut manager = self.manager.borrow_mut();
                    let resource = manager.resource_for(child_ptr);
                    let capabilities = self.cap_factory.build(&resource, self.manager.clone());
                    Ok(Some(Node::new(capabilities)))
                }
            }
        } else {
            Err(DoogieError::ResourceUnavailable)
        }
    }

    /// Get the last child Node of the current Node if any.
    pub fn last_child(&self) -> DoogieResult<Option<Node>> {
        if let Some(p) = *self.resource.node_pointer.borrow() {
            unsafe {
                let child_ptr = cmark_node_last_child(p);

                if child_ptr.is_null() {
                    Ok(None)
                } else {
                    let mut manager = self.manager.borrow_mut();
                    let resource = manager.resource_for(child_ptr);
                    let capabilities = self.cap_factory.build(&resource, self.manager.clone());
                    Ok(Some(Node::new(capabilities)))
                }
            }
        } else {
            Err(DoogieError::ResourceUnavailable)
        }
    }

    pub fn itself(&self) -> DoogieResult<Node> {
        if let Some(p) = *self.resource.node_pointer.borrow() {
            let mut manager = self.manager.borrow_mut();
            let resource = manager.resource_for(p);
            let capabilities = self.cap_factory.build(&resource, self.manager.clone());
            Ok(Node::new(capabilities))
        } else {
            Err(DoogieError::ResourceUnavailable)
        }
    }

    /// Get a NodeIterator based on the current node as root.
    pub fn iter(&self) -> NodeIterator {
        let mut capabilities = self.cap_factory.make_child_factory();
        capabilities.destructor_builder = None;

        match *self.resource.node_pointer.borrow() {
            Some(p) => unsafe {
                let iter_p = cmark_iter_new(p);
                NodeIterator::new(Some(iter_p), self.manager.clone(), capabilities)
            },
            _ => NodeIterator::new(None, self.manager.clone(), capabilities),
        }
    }
}

impl StructuralMutator {
    /// Collapse all empty adjacent text nodes.
    pub fn consolidate_text_nodes(&self) -> DoogieResult<()> {
        if let Some(p) = *self.resource.node_pointer.borrow() {
            unsafe {
                cmark_consolidate_text_nodes(p);
                Ok(())
            }
        } else {
            Err(DoogieError::ResourceUnavailable)
        }
    }

    /// Remove this Node from it's position in the document tree.
    ///
    /// After being unlinked, a Node will be the root of a new document tree whose resources
    /// are managed independently from the original tree. Responsibility for previously obtained
    /// instances of Nodes that are descendants of this Node will also be transferred to the new
    /// tree.
    pub fn unlink(&self) -> Node {
        if let Some(p) = *self.resource.node_pointer.borrow() {
            unsafe {
                cmark_node_unlink(p);
            }
        }

        let mut _manager = self.manager.borrow_mut();
        let new_manager = _manager.prune(&self.resource);

        let capabilities = self.cap_factory.build(&self.resource, new_manager.clone());
        Node::new(capabilities)
    }

    /// Add this Node as the last child of the given Node.
    ///
    /// If this Node was previously the root of its own document tree, responsibility for it and its
    /// children's resources will be transferred to the parent tree.
    pub fn append_child(&self, child: &mut Node) -> DoogieResult<u32> {
        let child_p: *mut CMarkNodePtr;
        let child_manager: SharedResourceMut<ResourceManager>;

        if let Some(ref mutator) = child.capabilities.mutate {
            child_manager = mutator.manager.clone();

            if let Some(p) = *mutator.resource.node_pointer.borrow() {
                child_p = p;
            } else {
                return Err(DoogieError::ResourceUnavailable);
            }
        } else {
            return Err(DoogieError::ResourceUnavailable);
        }

        if let Some(parent_p) = *self.resource.node_pointer.borrow() {
            unsafe {
                match cmark_node_append_child(parent_p, child_p) {
                    1 => {
                        let mut _manager = self.manager.borrow_mut();
                        _manager.absorb(&child_manager);
                        let child_resource = _manager.resource_for(child_p);
                        child.capabilities = self.cap_factory
                            .build(&child_resource, self.manager.clone());
                        Ok(1)
                    }
                    i => Err(DoogieError::ReturnCode(i as u32)),
                }
            }
        } else {
            Err(DoogieError::ResourceUnavailable)
        }
    }
}

impl NodeRenderer {
    /// Render the document tree rooted at the current node into textual CommonMark form.
    pub fn render_commonmark(&self) -> String {
        if let Some(node_pointer) = *self.resource.node_pointer.borrow() {
            unsafe {
                CStr::from_ptr(cmark_render_commonmark(node_pointer, 0))
                    .to_string_lossy()
                    .into_owned()
            }
        } else {
            String::new()
        }
    }

    pub fn render_xml(&self) -> String {
        if let Some(node_pointer) = *self.resource.node_pointer.borrow() {
            unsafe {
                CStr::from_ptr(cmark_render_xml(node_pointer, 0))
                    .to_string_lossy()
                    .into_owned()
            }
        } else {
            String::new()
        }
    }
}
