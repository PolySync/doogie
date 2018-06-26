#![feature(vec_remove_item)]

#[cfg(test)]
#[macro_use] extern crate proptest;
#[macro_use] extern crate log;

extern crate env_logger;
extern crate libc;
extern crate try_from;

pub mod errors;
pub mod constants;

use self::try_from::TryFrom;
use self::libc::{c_int, size_t, c_char, c_void};
use std::ffi::CStr;
use std::ffi::CString;
use std::rc::Rc;
use std::cell::RefCell;
use errors::DoogieError;
use constants::*;
use std::fmt::{
    Debug,
    Error,
    Formatter
};

/// Result type for the Doogie crate
pub type DoogieResult<T> = Result<T, DoogieError>;

/// Represents libcmark node pointers as an opaque struct
pub enum CMarkNodePtr {}
/// Represents libcmark iterator pointers as an opaque struct
enum CMarkIterPtr {}

extern "C" {
    fn cmark_node_new(node_type: u32) -> *mut CMarkNodePtr;

    fn cmark_parse_document(buffer: *const u8, len: size_t, options: c_int) -> *mut CMarkNodePtr;

    fn cmark_node_free(node: *mut CMarkNodePtr);

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

    fn cmark_iter_get_node(iter: *mut CMarkIterPtr) -> *mut CMarkNodePtr;

    fn cmark_iter_next(iter: *mut CMarkIterPtr) -> c_int;

    fn cmark_iter_free(iter: *mut CMarkIterPtr) -> c_void;
}

/// Contains the memory resource and management for a Node
#[derive(Clone)]
struct Resource {
    pub pointer: *mut CMarkNodePtr,
    manager: Rc<ResourceManager>,
}

impl Resource {
    /// Constructs a new `Resource` based on a libcmark Node Type
    fn from_node_type(node_type: NodeType, manager: Rc<ResourceManager>) -> Self {
        let pointer: *mut CMarkNodePtr;
        unsafe {
            pointer = cmark_node_new(node_type as u32);
        }
        Self {
            pointer,
            manager
        }
    }
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
        let manager = Rc::new(ResourceManager::new());
        manager.track_root(&root_ptr);

        Node::Document(Document {
            resource: Resource {
                pointer: root_ptr,
                manager,
            }
        })
    }
}

/// A node in the AST of a parsed commonmark document
pub enum Node {
    Document(Document),
    BlockQuote(BlockQuote),
    List(List),
    Item(Item),
    CodeBlock(CodeBlock),
    HtmlBlock(HtmlBlock),
    CustomBlock(CustomBlock),
    Paragraph(Paragraph),
    Heading(Heading),
    ThematicBreak(ThematicBreak),
    Text(Text),
    SoftBreak(SoftBreak),
    LineBreak(LineBreak),
    Code(Code),
    HtmlInline(HtmlInline),
    CustomInline(CustomInline),
    Emph(Emph),
    Strong(Strong),
    Link(Link),
    Image(Image)
}

impl PartialEq for Node {
    fn eq(&self, other: &Node) -> bool {
        self.as_resource().pointer == other.as_resource().pointer
    }
}

impl Debug for Node {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{} id: {:?}", self.get_cmark_type_string().unwrap_or("Type Unavailable".to_string()), self.as_resource().pointer)
    }
}

impl Node {
    /// Construct a Rust Node wrapper around a pointer to a libcmark node
    fn from_raw(pointer: *mut CMarkNodePtr) -> DoogieResult<Self> {
        let resource = Resource {
            pointer,
            manager: Rc::new(ResourceManager::new())
        };

        let cmark_type: NodeType;
        unsafe {
            cmark_type = NodeType::try_from(cmark_node_get_type(pointer) as u32)?;
        }

        match cmark_type {
            NodeType::CMarkNodeNone => Err(DoogieError::NodeNone),
            NodeType::CMarkNodeDocument => Ok(Node::Document(Document {resource})),
            NodeType::CMarkNodeList => Ok(Node::List(List {resource})),
            NodeType::CMarkNodeBlockQuote => Ok(Node::BlockQuote(BlockQuote {resource})),
            NodeType::CMarkNodeText => Ok(Node::Text(Text {resource})),
            NodeType::CMarkNodeParagraph => Ok(Node::Paragraph(Paragraph {resource})),
            NodeType::CMarkNodeItem => Ok(Node::Item(Item {resource})),
            NodeType::CMarkNodeCodeBlock => Ok(Node::CodeBlock(CodeBlock {resource})),
            NodeType::CMarkNodeHtmlBlock => Ok(Node::HtmlBlock(HtmlBlock {resource})),
            NodeType::CMarkNodeCustomBlock => Ok(Node::CustomBlock(CustomBlock {resource})),
            NodeType::CMarkNodeHeading => Ok(Node::Heading(Heading {resource})),
            NodeType::CMarkNodeThematicBreak => Ok(Node::ThematicBreak(ThematicBreak {resource})),
            NodeType::CMarkNodeSoftbreak => Ok(Node::SoftBreak(SoftBreak {resource})),
            NodeType::CMarkNodeLinebreak => Ok(Node::LineBreak(LineBreak {resource})),
            NodeType::CMarkNodeCode => Ok(Node::Code(Code {resource})),
            NodeType::CMarkNodeHtmlInline => Ok(Node::HtmlInline(HtmlInline {resource})),
            NodeType::CMarkNodeCustomInline => Ok(Node::CustomInline(CustomInline {resource})),
            NodeType::CMarkNodeEmph => Ok(Node::Emph(Emph {resource})),
            NodeType::CMarkNodeStrong => Ok(Node::Strong(Strong {resource})),
            NodeType::CMarkNodeLink => Ok(Node::Link(Link {resource})),
            NodeType::CMarkNodeImage => Ok(Node::Image(Image {resource})),
        }
    }

    /// Constructs a new `Node` of the given libcmark Node Type
    pub fn from_type(node_type: NodeType) -> DoogieResult<Self> {
        let pointer: *mut CMarkNodePtr;
        unsafe {
            pointer = cmark_node_new(node_type as u32);
        }
        Node::from_raw(pointer)
    }

    /// Returns the underlying resource object for this Node
    fn as_resource(&self) -> Resource {
        match self {
            &Node::Document(ref node) => node.resource.clone(),
            &Node::BlockQuote(ref node) => node.resource.clone(),
            &Node::Text(ref node) => node.resource.clone(),
            &Node::List(ref node) => node.resource.clone(),
            &Node::Item(ref node) => node.resource.clone(),
            &Node::CodeBlock(ref node) => node.resource.clone(),
            &Node::HtmlBlock(ref node) => node.resource.clone(),
            &Node::CustomBlock(ref node) => node.resource.clone(),
            &Node::Paragraph(ref node) => node.resource.clone(),
            &Node::Heading(ref node) => node.resource.clone(),
            &Node::ThematicBreak(ref node) => node.resource.clone(),
            &Node::SoftBreak(ref node) => node.resource.clone(),
            &Node::LineBreak(ref node) => node.resource.clone(),
            &Node::Code(ref node) => node.resource.clone(),
            &Node::HtmlInline(ref node) => node.resource.clone(),
            &Node::CustomInline(ref node) => node.resource.clone(),
            &Node::Emph(ref node) => node.resource.clone(),
            &Node::Strong(ref node) => node.resource.clone(),
            &Node::Link(ref node) => node.resource.clone(),
            &Node::Image(ref node) => node.resource.clone(),
        }
    }

    /// Returns the Rust equivalent of a libcmark NodeType enum
    pub fn get_cmark_type(&self) -> DoogieResult<NodeType> {
        unsafe {
            let t = cmark_node_get_type(self.as_resource().pointer);
            Ok(NodeType::try_from(t as u32)?)
        }
    }

    /// Returns a unique numerical identity for the `Node`
    pub fn get_id(&self) -> u32 {
        self.as_resource().pointer as u32
    }

    /// Returns a string version of the Node type
    pub fn get_cmark_type_string(&self) -> DoogieResult<String> {
        unsafe {
            let result = cmark_node_get_type_string(self.as_resource().pointer);
            if result.is_null() {
                warn!("Should not have gotten a nul pointer for node type string.");
                Ok(String::new())
            } else {
                Ok(CStr::from_ptr(result).to_str()?.to_string())
            }
        }
    }

    /// Returns the next sequential sibling of the current `Node` if it exists
    pub fn next_sibling(&self) -> DoogieResult<Option<Node>> {
        unsafe {
            let next_node_ptr = cmark_node_next(self.as_resource().pointer);

            if next_node_ptr.is_null() {
                Ok(None)
            } else {
                Ok(Some(Node::from_raw(next_node_ptr)?))
            }
        }
    }

    /// Returns the previous sequential sibling of the current `Node` if it exists
    pub fn prev_sibling(&self) -> DoogieResult<Option<Node>> {
        unsafe {
            let prev_node_ptr = cmark_node_previous(self.as_resource().pointer);

            if prev_node_ptr.is_null() {
                Ok(None)
            } else {
                Ok(Some(Node::from_raw(prev_node_ptr)?))
            }
        }
    }

    /// Returns the parent Node of the current `Node` if it exists
    pub fn parent(&self) -> DoogieResult<Option<Node>> {
        unsafe {
            let parent_node_ptr = cmark_node_parent(self.as_resource().pointer);

            if parent_node_ptr.is_null() {
                Ok(None)
            } else {
                Ok(Some(Node::from_raw(parent_node_ptr)?))
            }
        }
    }

    /// Returns the first child Node of the current `Node` if it exists
    pub fn first_child(&self) -> DoogieResult<Option<Node>> {
        unsafe {
            let child_ptr = cmark_node_first_child(self.as_resource().pointer);

            if child_ptr.is_null() {
                Ok(None)
            } else {
                Ok(Some(Node::from_raw(child_ptr)?))
            }
        }
    }

    /// Returns the last child Node of the current `Node` if it exists
    pub fn last_child(&self) -> DoogieResult<Option<Node>> {
        unsafe {
            let child_ptr = cmark_node_last_child(self.as_resource().pointer);

            if child_ptr.is_null() {
                Ok(None)
            } else {
                Ok(Some(Node::from_raw(child_ptr)?))
            }
        }
    }

    /// Returns a new instance of the current `Node`
    ///
    /// The returned `Node` will share the underlying memory resource and manager of the current Node.
    pub fn itself(&self) -> DoogieResult<Node> {
        Ok(Node::from_raw(self.as_resource().pointer)?)
    }


    /// Unlinks the current `Node` from its position in the document AST
    ///
    /// After unlinking, the Node will have no parent or siblings, but will retain all of its
    /// children.
    pub fn unlink(&self) {
        unsafe {
            cmark_node_unlink(self.as_resource().pointer);
        }
        self.as_resource().manager.track_root(&self.as_resource().pointer);
    }

    /// Append the given `Node` as the last child of the current `Node` if possible
    ///
    /// The rules of the CommonMark AST must be respected when appending nodes. Not all Nodes can
    /// be appended to each particular type of Node. Use `can_append_child` to determine if the
    /// operation will succeed. An error will be returned along with the libcmark error code if the
    /// operation.
    pub fn append_child(&self, child: &Node) -> DoogieResult<()> {
        child.unlink();
        unsafe {
            match cmark_node_append_child(self.as_resource().pointer, child.as_resource().pointer) {
                1 => {
                    child.as_resource().manager.untrack_root(&child.as_resource().pointer);
                    Ok(())
                }
                i => Err(DoogieError::ReturnCode(i as u32)),
            }
        }
    }

    /// Determines if the given `Node` is a potentially valid child of the current `Node`
    pub fn can_append_child(&self, child: &Node) -> DoogieResult<bool> {
        let child_type = child.get_cmark_type()?;

        let result = match self {
            Node::Document(_) => DOCUMENT_CHILDREN.contains(&child_type),
            Node::BlockQuote(_) => BLOCK_QUOTE_CHILDREN.contains(&child_type),
            Node::List(_) => child_type == NodeType::CMarkNodeItem,
            Node::Item(_) => ITEM_CHILDREN.contains(&child_type),
            Node::CodeBlock(_) => CODE_BLOCK_CHILDREN.contains(&child_type),
            Node::HtmlBlock(_) => HTML_BLOCK_CHILDREN.contains(&child_type),
            Node::CustomBlock(_) => CUSTOM_BLOCK_CHILDREN.contains(&child_type),
            Node::Paragraph(_) => PARAGRAPH_CHILDREN.contains(&child_type),
            Node::Heading(_) => HEADING_CHILDREN.contains(&child_type),
            Node::ThematicBreak(_) => THEMATIC_BREAK_CHILDREN.contains(&child_type),
            Node::Text(_) => TEXT_CHILDREN.contains(&child_type),
            Node::SoftBreak(_) => SOFT_BREAK_CHILDREN.contains(&child_type),
            Node::LineBreak(_) => LINE_BREAK_CHILDREN.contains(&child_type),
            Node::Code(_) => CODE_CHILDREN.contains(&child_type),
            Node::HtmlInline(_) => INLINE_HTML_CHILDREN.contains(&child_type),
            Node::CustomInline(_) => CUSTOM_INLINE_CHILDREN.contains(&child_type),
            Node::Emph(_) => EMPH_CHILDREN.contains(&child_type),
            Node::Strong(_) => STRONG_CHILDREN.contains(&child_type),
            Node::Link(_) => LINK_CHILDREN.contains(&child_type),
            Node::Image(_) => IMAGE_CHILDREN.contains(&child_type),
        };

        Ok(result)
    }

    /// Renders the document AST rooted at the current `Node` into textual CommonMark form
    pub fn render_commonmark(&self) -> String {
        unsafe {
            CStr::from_ptr(cmark_render_commonmark(self.as_resource().pointer, 0))
                .to_string_lossy()
                .into_owned()
        }
    }

    /// Renders the document AST rooted at the current `Node` into textual xml form
    pub fn render_xml(&self) -> String {
        unsafe {
            CStr::from_ptr(cmark_render_xml(self.as_resource().pointer, 0))
                .to_string_lossy()
                .into_owned()
        }
    }


    /// Returns an iterator over the `Node`s of the document subtree rooted at the current `Node`
    pub fn iter(&self) -> NodeIterator {
        let resource = self.as_resource();
        NodeIterator::new(resource.pointer)
    }

    /// Returns the start line from the original CMark document corresponding to the current `Node`
    pub fn get_start_line(&self) -> u32 {
        unsafe { cmark_node_get_start_line(self.as_resource().pointer) as u32 }
    }

    /// Returns the start column from the original CMark document corresponding to this `Node
    pub fn get_start_column(&self) -> u32 {
        unsafe { cmark_node_get_start_column(self.as_resource().pointer) as u32 }
    }
}

/// Represents the root `Node` of a document in the CommonMark AST
pub struct Document {
    resource: Resource,
}

impl Document {
    /// Constructs a new `Document`
    pub fn new() -> Self {
        Self {
            resource: Resource::from_node_type(NodeType::CMarkNodeDocument, Rc::new(ResourceManager::new()))
        }
    }

    /// Consolidates all adjacent `Text` `Node`s in the document into single `Text` `Node`s.
    pub fn consolidate_text_nodes(&self) {
        unsafe {
            cmark_consolidate_text_nodes(self.resource.pointer);
        }
    }
}

/// Represents a Block Quote element in CommonMark
pub struct BlockQuote {
    resource: Resource
}

impl BlockQuote {
    /// Constructs a new `BlockQuote`
    pub fn new() -> Self {
        Self {
            resource: Resource::from_node_type(NodeType::CMarkNodeBlockQuote, Rc::new(ResourceManager::new()))
        }
    }
}

/// Represents a List element in CommonMark
///
/// Lists are meta-containers in that they are classified as container blocks in CommonMark, but can
/// only contain `Item` elements as children.
pub struct List {
    resource: Resource
}

impl List {
    /// Constructs a new `List`
    pub fn new() -> Self {
        Self {
            resource: Resource::from_node_type(NodeType::CMarkNodeList, Rc::new(ResourceManager::new()))
        }
    }

    /// Returns an enum representing the type of list i.e. Bullet or Ordered
    pub fn get_list_type(&self) -> DoogieResult<ListType> {
        unsafe { ListType::try_from(cmark_node_get_list_type(self.resource.pointer) as u32) }
    }

    /// Returns the delimiter type used in the case of ordered lists.
    pub fn get_delim_type(&self) -> DoogieResult<DelimType> {
        unsafe { DelimType::try_from(cmark_node_get_list_delim(self.resource.pointer) as u32) }
    }
}

/// Represents a List Item in CommonMark
pub struct Item {
    resource: Resource
}

impl Item {
    /// Constructs a new `Item`
    pub fn new() -> Self {
        Self {
            resource: Resource::from_node_type(NodeType::CMarkNodeItem, Rc::new(ResourceManager::new())),
        }
    }
}

/// Represents a Code Block in CommonMark
pub struct CodeBlock {
    resource: Resource
}

impl CodeBlock {
    /// Constructs a new `CodeBlock`
    pub fn new() -> Self {
        Self {
            resource: Resource::from_node_type(NodeType::CMarkNodeCodeBlock, Rc::new(ResourceManager::new())),
        }
    }

    /// Returns the info text in the case of a Fenced Code Block
    pub fn get_fence_info(&self) -> DoogieResult<String> {
        unsafe {
            Ok(CStr::from_ptr(cmark_node_get_fence_info(self.resource.pointer))
                .to_str()?
                .to_string())
        }
    }

    /// Sets the info text for the code block
    pub fn set_fence_info(&self, info: &String) -> DoogieResult<u32> {
        unsafe {
            let info = CString::new(info.as_bytes())?;

            match cmark_node_set_fence_info(self.resource.pointer, info.as_ptr()) {
                1 => Ok(1),
                err => Err(DoogieError::ReturnCode(err as u32)),
            }
        }
    }

    /// Returns the textual content of the current Code Block element
    pub fn get_content(&self) -> DoogieResult<String> {
        unsafe {
            let result = cmark_node_get_literal(self.resource.pointer);
            if result.is_null() {
                return Ok(String::new());
            } else {
                return Ok(CStr::from_ptr(result).to_str()?.to_string());
            }
        }
    }

    /// Sets the textual content of the current Code Block element
    pub fn set_content(&self, content: &String) -> DoogieResult<u32> {
        unsafe {
            let content = CString::new(content.as_bytes())?;

            match cmark_node_set_literal(self.resource.pointer, content.as_ptr()) {
                1 => Ok(1 as u32),
                i => Err(DoogieError::ReturnCode(i as u32)),
            }
        }
    }

}

/// Represents a block of HTML in CommonMark
pub struct HtmlBlock {
    resource: Resource
}

impl HtmlBlock {
    /// Constructs a new `HtmlBlock`
    pub fn new() -> Self {
        Self {
            resource: Resource::from_node_type(NodeType::CMarkNodeHtmlBlock, Rc::new(ResourceManager::new())),
        }
    }
}

/// Represents an ambiguous Block Element
pub struct CustomBlock {
    resource: Resource
}

impl CustomBlock {
    /// Constructs a new `CustomBlock`
    pub fn new() -> Self {
        Self {
            resource: Resource::from_node_type(NodeType::CMarkNodeCustomBlock, Rc::new(ResourceManager::new())),
        }
    }
}

/// Represents a Paragraph element in CommonMark
pub struct Paragraph {
    resource: Resource
}

impl Paragraph {
    /// Constructs a new `Paragraph`
    pub fn new() -> Self {
        Self {
            resource: Resource::from_node_type(NodeType::CMarkNodeParagraph, Rc::new(ResourceManager::new())),
        }
    }
}

/// Represents a Heading element in CommonMark
pub struct Heading {
    resource: Resource
}

impl Heading {
    /// Constructs a new `Heading`
    pub fn new() -> Self {
        Self {
            resource: Resource::from_node_type(NodeType::CMarkNodeHeading, Rc::new(ResourceManager::new())),
        }
    }

    /// Returns the heading level of the current Heading
    pub fn get_level(&self) -> usize {
        unsafe { cmark_node_get_heading_level(self.resource.pointer) as usize }
    }
}

/// Represents a Thematic Break element in CommonMark
pub struct ThematicBreak {
    resource: Resource
}

impl ThematicBreak {
    /// Constructs a new `ThematicBreak`
    pub fn new() -> Self {
        Self {
            resource: Resource::from_node_type(NodeType::CMarkNodeThematicBreak, Rc::new(ResourceManager::new())),
        }
    }
}

/// Represents a Text element in CommonMark
pub struct Text {
    resource: Resource
}

impl Text {
    /// Constructs a new `Text`
    pub fn new() -> Self {
        Text {
            resource: Resource::from_node_type(NodeType::CMarkNodeText, Rc::new(ResourceManager::new())),
        }
    }

    /// Returns the textual content of the current Text element
    pub fn get_content(&self) -> DoogieResult<String> {
        unsafe {
            let result = cmark_node_get_literal(self.resource.pointer);
            if result.is_null() {
                return Ok(String::new());
            } else {
                return Ok(CStr::from_ptr(result).to_str()?.to_string());
            }
        }
    }

    /// Sets the textual content of the current Text element
    pub fn set_content(&self, content: &String) -> DoogieResult<u32> {
        unsafe {
            let content = CString::new(content.as_bytes())?;

            match cmark_node_set_literal(self.resource.pointer, content.as_ptr()) {
                1 => Ok(1 as u32),
                i => Err(DoogieError::ReturnCode(i as u32)),
            }
        }
    }
}

/// Represents a Soft Break element in CommonMark
pub struct SoftBreak {
    resource: Resource
}

impl SoftBreak {
    /// Constructs a new `SoftBreak`
    pub fn new() -> Self {
        Self {
            resource: Resource::from_node_type(NodeType::CMarkNodeSoftbreak, Rc::new(ResourceManager::new())),
        }
    }
}

/// Represents a Line Break element in CommonMark
pub struct LineBreak {
    resource: Resource
}

impl LineBreak {
    /// Constructs a new `LineBreak`
    pub fn new() -> Self {
        Self {
            resource: Resource::from_node_type(NodeType::CMarkNodeLinebreak, Rc::new(ResourceManager::new())),
        }
    }
}

/// Represents an inlin Code element in CommonMark
pub struct Code {
    resource: Resource
}

impl Code {
    /// Constructs a new `Code`
    pub fn new() -> Self {
        Self {
            resource: Resource::from_node_type(NodeType::CMarkNodeCode, Rc::new(ResourceManager::new())),
        }
    }

    /// Returns the textual content of the current Text element
    pub fn get_content(&self) -> DoogieResult<String> {
        unsafe {
            let result = cmark_node_get_literal(self.resource.pointer);
            if result.is_null() {
                return Ok(String::new());
            } else {
                return Ok(CStr::from_ptr(result).to_str()?.to_string());
            }
        }
    }

    /// Sets the textual content of the current Text element
    pub fn set_content(&self, content: &String) -> DoogieResult<u32> {
        unsafe {
            let content = CString::new(content.as_bytes())?;

            match cmark_node_set_literal(self.resource.pointer, content.as_ptr()) {
                1 => Ok(1 as u32),
                i => Err(DoogieError::ReturnCode(i as u32)),
            }
        }
    }
}

/// Represents an inline HTML element in CommonMark
pub struct HtmlInline {
    resource: Resource
}

impl HtmlInline {
    /// Constructs a new `HtmlInline`
    pub fn new() -> Self {
        Self {
            resource: Resource::from_node_type(NodeType::CMarkNodeHtmlInline, Rc::new(ResourceManager::new())),
        }
    }
}

/// Represents an ambiguous inline element
pub struct CustomInline {
    resource: Resource
}

impl CustomInline {
    /// Constructs a new `CustomInline`
    pub fn new() -> Self {
        Self {
            resource: Resource::from_node_type(NodeType::CMarkNodeCustomInline, Rc::new(ResourceManager::new())),
        }
    }
}

/// Represenets an Emph element in CommonMark
pub struct Emph {
    resource: Resource
}

impl Emph {
    /// Constructs a new `Emph`
    pub fn new() -> Self {
        Self {
            resource: Resource::from_node_type(NodeType::CMarkNodeEmph, Rc::new(ResourceManager::new())),
        }
    }
}

/// Represents a Strong element in CommonMark
pub struct Strong {
    resource: Resource
}

impl Strong {
    /// Constructs a new `Strong`
    pub fn new() -> Self {
        Self {
            resource: Resource::from_node_type(NodeType::CMarkNodeStrong, Rc::new(ResourceManager::new())),
        }
    }
}

/// Represents a Link element in CommonMark
pub struct Link {
    resource: Resource
}

impl Link {
    /// Constructs a new `Link`
    pub fn new() -> Self {
        Self {
            resource: Resource::from_node_type(NodeType::CMarkNodeLink, Rc::new(ResourceManager::new())),
        }
    }

    /// Returns the URL portion of the Link
    pub fn get_url(&self) -> DoogieResult<String> {
        unsafe {
            Ok(CStr::from_ptr(cmark_node_get_url(self.resource.pointer))
                .to_str()?
                .to_string())
        }
    }

    /// Returns the title portion of the Link
    pub fn get_title(&self) -> DoogieResult<String> {
        unsafe {
            Ok(CStr::from_ptr(cmark_node_get_title(self.resource.pointer))
                .to_str()?
                .to_string())
        }
    }

}

/// Represents an Image element in CommonMark
pub struct Image {
    resource: Resource
}

impl Image {
    /// Constructs a new `Image`
    pub fn new() -> Self {
        Self {
            resource: Resource::from_node_type(NodeType::CMarkNodeImage, Rc::new(ResourceManager::new())),
        }
    }
}

/// Iterator over the subtree rooted in the current node.
///
/// NodeIterator is a wrapper around the libcmark iterator and so traverses the subtree using the
/// same scheme documented [here](https://github.com/commonmark/cmark/blob/a5c83d7a426bda38aac838f9815664f6189d3404/src/cmark.h#L151).
pub struct NodeIterator {
    /// Raw CMark iterator pointer.
    pointer: *mut CMarkIterPtr,
}

impl NodeIterator {
    /// Construct a new instance.
    fn new(node_ptr: *mut CMarkNodePtr) -> NodeIterator {
        unsafe {
            NodeIterator {
                pointer: cmark_iter_new(node_ptr),
            }
        }
    }
}

impl Iterator for NodeIterator {
    type Item = (Node, IterEventType);

    /// Advance the iterator.
    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            match IterEventType::try_from(cmark_iter_next(self.pointer) as u32) {
                Ok(IterEventType::Done) | Ok(IterEventType::None) => None,
                Ok(event) => {
                    let node_p = cmark_iter_get_node(self.pointer);
                    match Node::from_raw(node_p) {
                        Ok(node) => Some((node, event)),
                        Err(_) => {
                            error!("Could not instantiate Node from Iterator.");
                            None
                        }
                    }
                }
                _ => None,
            }
        }
    }
}

impl Drop for NodeIterator {
    /// Free the CMark memory allocated for the iterator.
    fn drop(&mut self) {
        unsafe {
            cmark_iter_free(self.pointer);
        }
    }
}

/// Manages the memory resources of `Node` instances.
#[derive(Debug)]
struct ResourceManager {
    roots: RefCell<Vec<*mut CMarkNodePtr>>
}

impl Drop for ResourceManager {
    fn drop(&mut self) {
        let roots = self.roots.borrow();
        for pointer in roots.iter() {
            unsafe {
                cmark_node_free(*pointer);
            }
        }
    }
}

impl ResourceManager {
    /// Construct a new ResourceManager instance.
    pub fn new() -> ResourceManager {
        ResourceManager {
            roots: RefCell::new(Vec::new())
        }
    }

    /// Tracks the given pointer as a root Node of some tree or subtree
    pub fn track_root(&self, pointer: &*mut CMarkNodePtr) {
        let mut roots = self.roots.borrow_mut();
        if ! roots.contains(&pointer) {
            roots.push(pointer.clone());
        }
    }

    /// Removes the tracking for a given pointer
    pub fn untrack_root(&self, pointer: &*mut CMarkNodePtr) {
        let mut roots = self.roots.borrow_mut();
        roots.remove_item(pointer);
    }

    #[cfg(test)]
    /// Determines if the given pointer is currently being tracked
    pub fn is_tracking(&self, pointer: &*mut CMarkNodePtr) -> bool {
        let roots = self.roots.borrow();
        roots.contains(pointer)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        parse_document,
        Node,
        CodeBlock,
        Text,
        Resource,
        IterEventType,
        NodeType,
        cmark_node_new,
        CMarkNodePtr
    };
    use proptest::prelude::*;
    use constants::*;
    use try_from::TryFrom;

    /// Returns some arbitrary alphanumeric textual content
    fn arb_content(max_words: usize) -> BoxedStrategy<String> {
        prop::collection::vec("[[:alnum:]]{1,45}", 1..max_words)
            .prop_map(|v| v.join(" "))
            .boxed()
    }

    #[test]
    fn test_parse_document() {
        let body = "\
        # My New Document
        ";
        let node = parse_document(body);

        match node {
            Node::Document(_) => (),
            _ => panic!("Did not get a Document Node after parsing.")
        }
    }

    #[test]
    fn test_equality() {
        let body = "\
        # My New Document
        ";
        let node = parse_document(body);
        let other = node.itself().unwrap();

        assert_eq!(node, other);
    }

    #[test]
    fn test_inequality() {
        let body = "\
        # My New Document
        ";
        let node = parse_document(body);
        let other = node.first_child().unwrap().expect("Root should have a child");

        assert_ne!(node, other);
    }

    #[test]
    fn test_root_node_gets_tracked() {
        let body = "\
        # My New Document
        ";
        let resource: Resource;
        {
            let node = parse_document(body);
            resource = node.as_resource();
        }
        assert!(resource.manager.roots.borrow().contains(&resource.pointer));
    }

    #[test]
    fn test_iterator_hits_all_items() {
        let body = "* Item 1\n* Item 2\n* Item 3";
        let root = parse_document(body);
        let mut node_contents: Vec<String> = Vec::new();
        let mut item_count = 0;

        for item in root.iter() {
            match item {
                (Node::Item(_), IterEventType::Enter) => item_count += 1,
                (Node::Text(ref text), IterEventType::Enter) => node_contents.push(text.get_content().unwrap()),
                _ => ()
            }
        }

        assert_eq!(item_count, 3);
        assert!(node_contents.contains(&String::from("Item 1")));
        assert!(node_contents.contains(&String::from("Item 2")));
        assert!(node_contents.contains(&String::from("Item 3")));
    }

    #[test]
    fn test_parent_child_traversal() {
        let body = "* Item 1\n* Item 2\n* Item 3";
        let root = parse_document(body);
        let child = root.first_child().unwrap().expect("Root should have had child");
        assert_eq!(root, child.parent().unwrap().expect("Child should have had a parent"));
    }

    #[test]
    fn test_sibling_traversal() {
        let body = "* Item 1\n* Item 2\n* Item 3";
        let root = parse_document(body);
        let list = root.first_child().unwrap().expect("Root should have had list");
        let first_item = list.first_child().unwrap().expect("List should have had item");
        let next_item = first_item.next_sibling().unwrap().expect("First item should have had next sibling");

        assert_eq!(first_item, next_item.prev_sibling().unwrap().expect("Next item should have had prev item"));
    }

    #[test]
    fn parse_and_render() {
        let content = "# Testing";
        let root = parse_document(content);

        assert_eq!(content, root.render_commonmark().trim());
    }

    #[test]
    fn test_from_raw() {
        let node_pointer: *mut CMarkNodePtr;
        unsafe {
            node_pointer = cmark_node_new(NodeType::CMarkNodeParagraph as u32);
        }

        let node = Node::from_raw(node_pointer).unwrap();

        match node {
            Node::Paragraph(_) => (),
            _ => panic!("Node should have been a paragraph")
        }
    }

    #[test]
    fn test_unlink() {
        let body = "* Item 1\n* Item 2\n* Item 3";
        let root = parse_document(body);
        let first_item = root
            .first_child()
            .unwrap().expect("Root should have first child")
            .first_child()
            .unwrap().expect("List should have first item");
        let manager = first_item.as_resource().manager;

        first_item.unlink();

        assert!(manager.roots.borrow().contains(&first_item.as_resource().pointer));
        for (node, _) in root.iter() {
            if let Node::Text(node) = node {
                assert!(! node.get_content().unwrap().contains("Item 1"));
            }
        }
    }

    #[test]
    fn test_append_child() {
        let root_node = Node::from_type(NodeType::CMarkNodeDocument).unwrap();
        let child_node = Node::from_type(NodeType::CMarkNodeParagraph).unwrap();

        root_node.append_child(&child_node).unwrap();

        assert!( ! root_node.as_resource().manager.is_tracking(&child_node.as_resource().pointer));
        assert_eq!(root_node.first_child().unwrap().expect("Root should have child"), child_node);
    }

    #[test]
    fn test_document_children() {
        for i in 1..21 {
            let node_type = NodeType::CMarkNodeDocument;
            let other_type = NodeType::try_from(i).unwrap();
            let node = Node::from_type(node_type).unwrap();
            let child = Node::from_type(other_type.clone()).unwrap();
            match node.can_append_child(&child).unwrap() {
                true => assert!(DOCUMENT_CHILDREN.contains(&other_type), "{:?} should not have been a valid block quote child, but was", other_type),
                false => assert!( ! DOCUMENT_CHILDREN.contains(&other_type), "{:?} should be a valid block quote child, but was not", other_type)
            }
            match node.append_child(&child) {
                Err(_) => assert!( ! DOCUMENT_CHILDREN.contains(&other_type), "{:?} should be able to append, but was not", other_type),
                Ok(_) => assert!(DOCUMENT_CHILDREN.contains(&other_type), "{:?} should not have been able to append, but was", other_type)
            }
        }
    }

    #[test]
    fn test_block_quote_children() {
        for i in 1..21 {
            let node_type = NodeType::CMarkNodeBlockQuote;
            let other_type = NodeType::try_from(i).unwrap();
            let node = Node::from_type(node_type).unwrap();
            let child = Node::from_type(other_type.clone()).unwrap();
            match node.can_append_child(&child).unwrap() {
                true => assert!(BLOCK_QUOTE_CHILDREN.contains(&other_type), "{:?} should not have been a valid block quote child, but was", other_type),
                false => assert!( ! BLOCK_QUOTE_CHILDREN.contains(&other_type), "{:?} should be a valid block quote child, but was not", other_type)
            }
            match node.append_child(&child) {
                Err(_) => assert!( ! BLOCK_QUOTE_CHILDREN.contains(&other_type), "{:?} should be able to append, but was not", other_type),
                Ok(_) => assert!(BLOCK_QUOTE_CHILDREN.contains(&other_type), "{:?} should not have been able to append, but was", other_type)
            }
        }
    }

    #[test]
    fn test_list_children() {
        for i in 1..21 {
            let node_type = NodeType::CMarkNodeList;
            let other_type = NodeType::try_from(i).unwrap();
            let node = Node::from_type(node_type).unwrap();
            let child = Node::from_type(other_type.clone()).unwrap();
            match node.can_append_child(&child).unwrap() {
                true => assert!(LIST_CHILDREN.contains(&other_type), "{:?} should not have been a valid block quote child, but was", other_type),
                false => assert!( ! LIST_CHILDREN.contains(&other_type), "{:?} should be a valid block quote child, but was not", other_type)
            }
            match node.append_child(&child) {
                Err(_) => assert!( ! LIST_CHILDREN.contains(&other_type), "{:?} should be able to append, but was not", other_type),
                Ok(_) => assert!(LIST_CHILDREN.contains(&other_type), "{:?} should not have been able to append, but was", other_type)
            }
        }
    }

    #[test]
    fn test_item_children() {
        for i in 1..21 {
            let node_type = NodeType::CMarkNodeItem;
            let other_type = NodeType::try_from(i).unwrap();
            let node = Node::from_type(node_type).unwrap();
            let child = Node::from_type(other_type.clone()).unwrap();
            match node.can_append_child(&child).unwrap() {
                true => assert!(ITEM_CHILDREN.contains(&other_type), "{:?} should not have been a valid block quote child, but was", other_type),
                false => assert!( ! ITEM_CHILDREN.contains(&other_type), "{:?} should be a valid block quote child, but was not", other_type)
            }
            match node.append_child(&child) {
                Err(_) => assert!( ! ITEM_CHILDREN.contains(&other_type), "{:?} should be able to append, but was not", other_type),
                Ok(_) => assert!(ITEM_CHILDREN.contains(&other_type), "{:?} should not have been able to append, but was", other_type)
            }
        }
    }

    #[test]
    fn test_code_block_children() {
        for i in 1..21 {
            let node_type = NodeType::CMarkNodeCodeBlock;
            let other_type = NodeType::try_from(i).unwrap();
            let node = Node::from_type(node_type).unwrap();
            let child = Node::from_type(other_type.clone()).unwrap();
            match node.can_append_child(&child).unwrap() {
                true => assert!(CODE_BLOCK_CHILDREN.contains(&other_type), "{:?} should not have been a valid block quote child, but was", other_type),
                false => assert!( ! CODE_BLOCK_CHILDREN.contains(&other_type), "{:?} should be a valid block quote child, but was not", other_type)
            }
            match node.append_child(&child) {
                Err(_) => assert!( ! CODE_BLOCK_CHILDREN.contains(&other_type), "{:?} should be able to append, but was not", other_type),
                Ok(_) => assert!(CODE_BLOCK_CHILDREN.contains(&other_type), "{:?} should not have been able to append, but was", other_type)
            }
        }
    }

    #[test]
    fn test_html_block_children() {
        for i in 1..21 {
            let node_type = NodeType::CMarkNodeHtmlBlock;
            let other_type = NodeType::try_from(i).unwrap();
            let node = Node::from_type(node_type).unwrap();
            let child = Node::from_type(other_type.clone()).unwrap();
            match node.can_append_child(&child).unwrap() {
                true => assert!(HTML_BLOCK_CHILDREN.contains(&other_type), "{:?} should not have been a valid block quote child, but was", other_type),
                false => assert!( ! HTML_BLOCK_CHILDREN.contains(&other_type), "{:?} should be a valid block quote child, but was not", other_type)
            }
            match node.append_child(&child) {
                Err(_) => assert!( ! HTML_BLOCK_CHILDREN.contains(&other_type), "{:?} should be able to append, but was not", other_type),
                Ok(_) => assert!(HTML_BLOCK_CHILDREN.contains(&other_type), "{:?} should not have been able to append, but was", other_type)
            }
        }
    }

    #[test]
    fn test_custom_block_children() {
        for i in 1..21 {
            let node_type = NodeType::CMarkNodeCustomBlock;
            let other_type = NodeType::try_from(i).unwrap();
            let node = Node::from_type(node_type).unwrap();
            let child = Node::from_type(other_type.clone()).unwrap();
            match node.can_append_child(&child).unwrap() {
                true => assert!(CUSTOM_BLOCK_CHILDREN.contains(&other_type), "{:?} should not have been a valid block quote child, but was", other_type),
                false => assert!( ! CUSTOM_BLOCK_CHILDREN.contains(&other_type), "{:?} should be a valid block quote child, but was not", other_type)
            }
            match node.append_child(&child) {
                Err(_) => assert!( ! CUSTOM_BLOCK_CHILDREN.contains(&other_type), "{:?} should be able to append, but was not", other_type),
                Ok(_) => assert!(CUSTOM_BLOCK_CHILDREN.contains(&other_type), "{:?} should not have been able to append, but was", other_type)
            }
        }
    }

    #[test]
    fn test_paragraph_children() {
        for i in 1..21 {
            let node_type = NodeType::CMarkNodeParagraph;
            let other_type = NodeType::try_from(i).unwrap();
            let node = Node::from_type(node_type).unwrap();
            let child = Node::from_type(other_type.clone()).unwrap();
            match node.can_append_child(&child).unwrap() {
                true => assert!(PARAGRAPH_CHILDREN.contains(&other_type), "{:?} should not have been a valid block quote child, but was", other_type),
                false => assert!( ! PARAGRAPH_CHILDREN.contains(&other_type), "{:?} should be a valid block quote child, but was not", other_type)
            }
            match node.append_child(&child) {
                Err(_) => assert!( ! PARAGRAPH_CHILDREN.contains(&other_type), "{:?} should be able to append, but was not", other_type),
                Ok(_) => assert!(PARAGRAPH_CHILDREN.contains(&other_type), "{:?} should not have been able to append, but was", other_type)
            }
        }
    }

    #[test]
    fn test_heading_children() {
        for i in 1..21 {
            let node_type = NodeType::CMarkNodeHeading;
            let other_type = NodeType::try_from(i).unwrap();
            let node = Node::from_type(node_type).unwrap();
            let child = Node::from_type(other_type.clone()).unwrap();
            match node.can_append_child(&child).unwrap() {
                true => assert!(HEADING_CHILDREN.contains(&other_type), "{:?} should not have been a valid block quote child, but was", other_type),
                false => assert!( ! HEADING_CHILDREN.contains(&other_type), "{:?} should be a valid block quote child, but was not", other_type)
            }
            match node.append_child(&child) {
                Err(_) => assert!( ! HEADING_CHILDREN.contains(&other_type), "{:?} should be able to append, but was not", other_type),
                Ok(_) => assert!(HEADING_CHILDREN.contains(&other_type), "{:?} should not have been able to append, but was", other_type)
            }
        }
    }


    #[test]
    fn test_thematic_break_children() {
        for i in 1..21 {
            let node_type = NodeType::CMarkNodeThematicBreak;
            let other_type = NodeType::try_from(i).unwrap();
            let node = Node::from_type(node_type).unwrap();
            let child = Node::from_type(other_type.clone()).unwrap();
            match node.can_append_child(&child).unwrap() {
                true => assert!(THEMATIC_BREAK_CHILDREN.contains(&other_type), "{:?} should not have been a valid block quote child, but was", other_type),
                false => assert!( ! THEMATIC_BREAK_CHILDREN.contains(&other_type), "{:?} should be a valid block quote child, but was not", other_type)
            }
            match node.append_child(&child) {
                Err(_) => assert!( ! THEMATIC_BREAK_CHILDREN.contains(&other_type), "{:?} should be able to append, but was not", other_type),
                Ok(_) => assert!(THEMATIC_BREAK_CHILDREN.contains(&other_type), "{:?} should not have been able to append, but was", other_type)
            }
        }
    }

    #[test]
    fn test_text_children() {
        for i in 1..21 {
            let node_type = NodeType::CMarkNodeText;
            let other_type = NodeType::try_from(i).unwrap();
            let node = Node::from_type(node_type).unwrap();
            let child = Node::from_type(other_type.clone()).unwrap();
            match node.can_append_child(&child).unwrap() {
                true => assert!(TEXT_CHILDREN.contains(&other_type), "{:?} should not have been a valid block quote child, but was", other_type),
                false => assert!( ! TEXT_CHILDREN.contains(&other_type), "{:?} should be a valid block quote child, but was not", other_type)
            }
            match node.append_child(&child) {
                Err(_) => assert!( ! TEXT_CHILDREN.contains(&other_type), "{:?} should be able to append, but was not", other_type),
                Ok(_) => assert!(TEXT_CHILDREN.contains(&other_type), "{:?} should not have been able to append, but was", other_type)
            }
        }
    }

    #[test]
    fn test_soft_break_children() {
        for i in 1..21 {
            let node_type = NodeType::CMarkNodeSoftbreak;
            let other_type = NodeType::try_from(i).unwrap();
            let node = Node::from_type(node_type).unwrap();
            let child = Node::from_type(other_type.clone()).unwrap();
            match node.can_append_child(&child).unwrap() {
                true => assert!(SOFT_BREAK_CHILDREN.contains(&other_type), "{:?} should not have been a valid block quote child, but was", other_type),
                false => assert!( ! SOFT_BREAK_CHILDREN.contains(&other_type), "{:?} should be a valid block quote child, but was not", other_type)
            }
            match node.append_child(&child) {
                Err(_) => assert!( ! SOFT_BREAK_CHILDREN.contains(&other_type), "{:?} should be able to append, but was not", other_type),
                Ok(_) => assert!(SOFT_BREAK_CHILDREN.contains(&other_type), "{:?} should not have been able to append, but was", other_type)
            }
        }
    }

    #[test]
    fn test_line_break_children() {
        for i in 1..21 {
            let node_type = NodeType::CMarkNodeLinebreak;
            let other_type = NodeType::try_from(i).unwrap();
            let node = Node::from_type(node_type).unwrap();
            let child = Node::from_type(other_type.clone()).unwrap();
            match node.can_append_child(&child).unwrap() {
                true => assert!(LINE_BREAK_CHILDREN.contains(&other_type), "{:?} should not have been a valid block quote child, but was", other_type),
                false => assert!( ! LINE_BREAK_CHILDREN.contains(&other_type), "{:?} should be a valid block quote child, but was not", other_type)
            }
            match node.append_child(&child) {
                Err(_) => assert!( ! LINE_BREAK_CHILDREN.contains(&other_type), "{:?} should be able to append, but was not", other_type),
                Ok(_) => assert!(LINE_BREAK_CHILDREN.contains(&other_type), "{:?} should not have been able to append, but was", other_type)
            }
        }
    }

    #[test]
    fn test_code_children() {
        for i in 1..21 {
            let node_type = NodeType::CMarkNodeCode;
            let other_type = NodeType::try_from(i).unwrap();
            let node = Node::from_type(node_type).unwrap();
            let child = Node::from_type(other_type.clone()).unwrap();
            match node.can_append_child(&child).unwrap() {
                true => assert!(CODE_CHILDREN.contains(&other_type), "{:?} should not have been a valid block quote child, but was", other_type),
                false => assert!( ! CODE_CHILDREN.contains(&other_type), "{:?} should be a valid block quote child, but was not", other_type)
            }
            match node.append_child(&child) {
                Err(_) => assert!( ! CODE_CHILDREN.contains(&other_type), "{:?} should be able to append, but was not", other_type),
                Ok(_) => assert!(CODE_CHILDREN.contains(&other_type), "{:?} should not have been able to append, but was", other_type)
            }
        }
    }

    #[test]
    fn test_inline_html_children() {
        for i in 1..21 {
            let node_type = NodeType::CMarkNodeHtmlInline;
            let other_type = NodeType::try_from(i).unwrap();
            let node = Node::from_type(node_type).unwrap();
            let child = Node::from_type(other_type.clone()).unwrap();
            match node.can_append_child(&child).unwrap() {
                true => assert!(INLINE_HTML_CHILDREN.contains(&other_type), "{:?} should not have been a valid block quote child, but was", other_type),
                false => assert!( ! INLINE_HTML_CHILDREN.contains(&other_type), "{:?} should be a valid block quote child, but was not", other_type)
            }
            match node.append_child(&child) {
                Err(_) => assert!( ! INLINE_HTML_CHILDREN.contains(&other_type), "{:?} should be able to append, but was not", other_type),
                Ok(_) => assert!(INLINE_HTML_CHILDREN.contains(&other_type), "{:?} should not have been able to append, but was", other_type)
            }
        }
    }


    #[test]
    fn test_custom_inline_children() {
        for i in 1..21 {
            let node_type = NodeType::CMarkNodeCustomInline;
            let other_type = NodeType::try_from(i).unwrap();
            let node = Node::from_type(node_type).unwrap();
            let child = Node::from_type(other_type.clone()).unwrap();
            match node.can_append_child(&child).unwrap() {
                true => assert!(CUSTOM_INLINE_CHILDREN.contains(&other_type), "{:?} should not have been a valid block quote child, but was", other_type),
                false => assert!( ! CUSTOM_INLINE_CHILDREN.contains(&other_type), "{:?} should be a valid block quote child, but was not", other_type)
            }
            match node.append_child(&child) {
                Err(_) => assert!( ! CUSTOM_INLINE_CHILDREN.contains(&other_type), "{:?} should be able to append, but was not", other_type),
                Ok(_) => assert!(CUSTOM_INLINE_CHILDREN.contains(&other_type), "{:?} should not have been able to append, but was", other_type)
            }
        }
    }

    #[test]
    fn test_emph_children() {
        for i in 1..21 {
            let node_type = NodeType::CMarkNodeEmph;
            let other_type = NodeType::try_from(i).unwrap();
            let node = Node::from_type(node_type).unwrap();
            let child = Node::from_type(other_type.clone()).unwrap();
            match node.can_append_child(&child).unwrap() {
                true => assert!(EMPH_CHILDREN.contains(&other_type), "{:?} should not have been a valid block quote child, but was", other_type),
                false => assert!( ! EMPH_CHILDREN.contains(&other_type), "{:?} should be a valid block quote child, but was not", other_type)
            }
            match node.append_child(&child) {
                Err(_) => assert!( ! EMPH_CHILDREN.contains(&other_type), "{:?} should be able to append, but was not", other_type),
                Ok(_) => assert!(EMPH_CHILDREN.contains(&other_type), "{:?} should not have been able to append, but was", other_type)
            }
        }
    }

    #[test]
    fn test_strong_children() {
        for i in 1..21 {
            let node_type = NodeType::CMarkNodeStrong;
            let other_type = NodeType::try_from(i).unwrap();
            let node = Node::from_type(node_type).unwrap();
            let child = Node::from_type(other_type.clone()).unwrap();
            match node.can_append_child(&child).unwrap() {
                true => assert!(STRONG_CHILDREN.contains(&other_type), "{:?} should not have been a valid block quote child, but was", other_type),
                false => assert!( ! STRONG_CHILDREN.contains(&other_type), "{:?} should be a valid block quote child, but was not", other_type)
            }
            match node.append_child(&child) {
                Err(_) => assert!( ! STRONG_CHILDREN.contains(&other_type), "{:?} should be able to append, but was not", other_type),
                Ok(_) => assert!(STRONG_CHILDREN.contains(&other_type), "{:?} should not have been able to append, but was", other_type)
            }
        }
    }

    #[test]
    fn test_link_children() {
        for i in 1..21 {
            let node_type = NodeType::CMarkNodeLink;
            let other_type = NodeType::try_from(i).unwrap();
            let node = Node::from_type(node_type).unwrap();
            let child = Node::from_type(other_type.clone()).unwrap();
            match node.can_append_child(&child).unwrap() {
                true => assert!(LINK_CHILDREN.contains(&other_type), "{:?} should not have been a valid block quote child, but was", other_type),
                false => assert!( ! LINK_CHILDREN.contains(&other_type), "{:?} should be a valid block quote child, but was not", other_type)
            }
            match node.append_child(&child) {
                Err(_) => assert!( ! LINK_CHILDREN.contains(&other_type), "{:?} should be able to append, but was not", other_type),
                Ok(_) => assert!(LINK_CHILDREN.contains(&other_type), "{:?} should not have been able to append, but was", other_type)
            }
        }
    }

    #[test]
    fn test_image_children() {
        for i in 1..21 {
            let node_type = NodeType::CMarkNodeImage;
            let other_type = NodeType::try_from(i).unwrap();
            let node = Node::from_type(node_type).unwrap();
            let child = Node::from_type(other_type.clone()).unwrap();
            match node.can_append_child(&child).unwrap() {
                true => assert!(IMAGE_CHILDREN.contains(&other_type), "{:?} should not have been a valid block quote child, but was", other_type),
                false => assert!( ! IMAGE_CHILDREN.contains(&other_type), "{:?} should be a valid block quote child, but was not", other_type)
            }
            match node.append_child(&child) {
                Err(_) => assert!( ! IMAGE_CHILDREN.contains(&other_type), "{:?} should be able to append, but was not", other_type),
                Ok(_) => assert!(IMAGE_CHILDREN.contains(&other_type), "{:?} should not have been able to append, but was", other_type)
            }
        }
    }

    proptest! {
        #[test]
        fn test_text_set_and_get_content(ref content in arb_content(10)) {
                let text_node = Text::new();
                text_node.set_content(content).unwrap();
                assert_eq!(content, &text_node.get_content().unwrap());
        }
    }

    proptest! {
        #[test]
        fn test_fence_info_get_set(ref content in arb_content(10)){
            let node = CodeBlock::new();
            node.set_fence_info(content).unwrap();
            assert_eq!(content, &node.get_fence_info().unwrap());
        }
    }
}
