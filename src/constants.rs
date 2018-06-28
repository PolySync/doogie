extern crate try_from;

use self::try_from::TryFrom;
use super::{DoogieError, DoogieResult};
use std::collections::HashSet;

/// Each NodeIterator step is parameterized by one of these event.
#[derive(PartialEq, Debug)]
pub enum IterEventType {
    None,
    /// The iteration has reached the end.
    Done,
    /// The current Node is being descended into.
    Enter,
    /// The current Node is being exited out of.
    Exit,
}

impl From<IterEventType> for u32 {
    fn from(original: IterEventType) -> u32 {
        match original {
            IterEventType::None => 0,
            IterEventType::Done => 1,
            IterEventType::Enter => 2,
            IterEventType::Exit => 3,
        }
    }
}

impl TryFrom<u32> for IterEventType {
    type Err = DoogieError;

    fn try_from(original: u32) -> DoogieResult<IterEventType> {
        match original {
            0 => Ok(IterEventType::None),
            1 => Ok(IterEventType::Done),
            2 => Ok(IterEventType::Enter),
            3 => Ok(IterEventType::Exit),
            i => Err(DoogieError::BadEnum(i)),
        }
    }
}

/// Each Node in the libcmark document AST possesses a type attribute that corresponds to its
/// equivalent CommonMark semantic element.
#[derive(PartialEq, Debug, Clone, Eq, Hash)]
pub enum NodeType {
    CMarkNodeNone,
    CMarkNodeDocument,
    CMarkNodeBlockQuote,
    CMarkNodeList,
    CMarkNodeItem,
    CMarkNodeCodeBlock,
    CMarkNodeHtmlBlock,
    CMarkNodeCustomBlock,
    CMarkNodeParagraph,
    CMarkNodeHeading,
    CMarkNodeThematicBreak,
    CMarkNodeText,
    CMarkNodeSoftbreak,
    CMarkNodeLinebreak,
    CMarkNodeCode,
    CMarkNodeHtmlInline,
    CMarkNodeCustomInline,
    CMarkNodeEmph,
    CMarkNodeStrong,
    CMarkNodeLink,
    CMarkNodeImage,
}

impl From<NodeType> for u32 {
    fn from(node_type: NodeType) -> Self {
        match node_type {
            NodeType::CMarkNodeNone => 0,
            NodeType::CMarkNodeDocument => 1,
            NodeType::CMarkNodeBlockQuote => 2,
            NodeType::CMarkNodeList => 3,
            NodeType::CMarkNodeItem => 4,
            NodeType::CMarkNodeCodeBlock => 5,
            NodeType::CMarkNodeHtmlBlock => 6,
            NodeType::CMarkNodeCustomBlock => 7,
            NodeType::CMarkNodeParagraph => 8,
            NodeType::CMarkNodeHeading => 9,
            NodeType::CMarkNodeThematicBreak => 10,
            NodeType::CMarkNodeText => 11,
            NodeType::CMarkNodeSoftbreak => 12,
            NodeType::CMarkNodeLinebreak => 13,
            NodeType::CMarkNodeCode => 14,
            NodeType::CMarkNodeHtmlInline => 15,
            NodeType::CMarkNodeCustomInline => 16,
            NodeType::CMarkNodeEmph => 17,
            NodeType::CMarkNodeStrong => 18,
            NodeType::CMarkNodeLink => 19,
            NodeType::CMarkNodeImage => 20,
        }
    }
}

impl TryFrom<u32> for NodeType {
    type Err = DoogieError;

    fn try_from(original: u32) -> DoogieResult<Self> {
        match original {
            0 => Ok(NodeType::CMarkNodeNone),
            1 => Ok(NodeType::CMarkNodeDocument),
            2 => Ok(NodeType::CMarkNodeBlockQuote),
            3 => Ok(NodeType::CMarkNodeList),
            4 => Ok(NodeType::CMarkNodeItem),
            5 => Ok(NodeType::CMarkNodeCodeBlock),
            6 => Ok(NodeType::CMarkNodeHtmlBlock),
            7 => Ok(NodeType::CMarkNodeCustomBlock),
            8 => Ok(NodeType::CMarkNodeParagraph),
            9 => Ok(NodeType::CMarkNodeHeading),
            10 => Ok(NodeType::CMarkNodeThematicBreak),
            11 => Ok(NodeType::CMarkNodeText),
            12 => Ok(NodeType::CMarkNodeSoftbreak),
            13 => Ok(NodeType::CMarkNodeLinebreak),
            14 => Ok(NodeType::CMarkNodeCode),
            15 => Ok(NodeType::CMarkNodeHtmlInline),
            16 => Ok(NodeType::CMarkNodeCustomInline),
            17 => Ok(NodeType::CMarkNodeEmph),
            18 => Ok(NodeType::CMarkNodeStrong),
            19 => Ok(NodeType::CMarkNodeLink),
            20 => Ok(NodeType::CMarkNodeImage),
            i => Err(DoogieError::BadEnum(i)),
        }
    }
}

/// List elements have one of these types associated with them
#[derive(PartialEq)]
pub enum ListType {
    CMarkNoList,
    CMarkBulletList,
    CMarkOrderedList,
}

impl From<ListType> for u32 {
    fn from(original: ListType) -> u32 {
        match original {
            ListType::CMarkNoList => 0,
            ListType::CMarkBulletList => 1,
            ListType::CMarkOrderedList => 2,
        }
    }
}

impl TryFrom<u32> for ListType {
    type Err = DoogieError;

    fn try_from(original: u32) -> DoogieResult<Self> {
        match original {
            0 => Ok(ListType::CMarkNoList),
            1 => Ok(ListType::CMarkBulletList),
            2 => Ok(ListType::CMarkOrderedList),
            i => Err(DoogieError::BadEnum(i)),
        }
    }
}

/// Ordered List items have a delimiter attribute
#[derive(PartialEq)]
pub enum DelimType {
    CMarkNoDelim,
    CMarkPeriodDelim,
    CMarkParenDelim,
}

impl From<DelimType> for u32 {
    fn from(original: DelimType) -> u32 {
        match original {
            DelimType::CMarkNoDelim => 0,
            DelimType::CMarkPeriodDelim => 1,
            DelimType::CMarkParenDelim => 2,
        }
    }
}

impl TryFrom<u32> for DelimType {
    type Err = DoogieError;

    fn try_from(original: u32) -> DoogieResult<DelimType> {
        match original {
            0 => Ok(DelimType::CMarkNoDelim),
            1 => Ok(DelimType::CMarkPeriodDelim),
            2 => Ok(DelimType::CMarkParenDelim),
            i => Err(DoogieError::BadEnum(i)),
        }
    }
}

/// Valid child types of Document elements
lazy_static! {
    pub static ref DOCUMENT_CHILDREN: HashSet<NodeType> = {
        let mut children = HashSet::new();
        children.insert(NodeType::CMarkNodeParagraph);
        children.insert(NodeType::CMarkNodeHeading);
        children.insert(NodeType::CMarkNodeThematicBreak);
        children.insert(NodeType::CMarkNodeCodeBlock);
        children.insert(NodeType::CMarkNodeHtmlBlock);
        children.insert(NodeType::CMarkNodeCustomBlock);
        children.insert(NodeType::CMarkNodeList);
        children.insert(NodeType::CMarkNodeBlockQuote);
        children
    };
}

/// Valid child types of List elements
lazy_static! {
    pub static ref LIST_CHILDREN: HashSet<NodeType> = {
        let mut children = HashSet::new();
        children.insert(NodeType::CMarkNodeItem);
        children
    };
}

/// Valid child types of List Item elements
lazy_static! {
    pub static ref ITEM_CHILDREN: HashSet<NodeType> = { DOCUMENT_CHILDREN.clone() };
}

/// Valid child types of Block Quote elements
lazy_static! {
    pub static ref BLOCK_QUOTE_CHILDREN: HashSet<NodeType> = { DOCUMENT_CHILDREN.clone() };
}

/// Valid child types of Code Block elements
lazy_static! {
    pub static ref CODE_BLOCK_CHILDREN: HashSet<NodeType> = { HashSet::new() };
}

/// Valid child types of HTML Block elements
lazy_static! {
    pub static ref HTML_BLOCK_CHILDREN: HashSet<NodeType> = { HashSet::new() };
}

/// Valid child types of Custom Block elements
lazy_static! {
    pub static ref CUSTOM_BLOCK_CHILDREN: HashSet<NodeType> = {
        let mut children = HashSet::new();
        children.insert(NodeType::CMarkNodeBlockQuote);
        children.insert(NodeType::CMarkNodeList);
        children.insert(NodeType::CMarkNodeItem);
        children.insert(NodeType::CMarkNodeCodeBlock);
        children.insert(NodeType::CMarkNodeHtmlBlock);
        children.insert(NodeType::CMarkNodeCustomBlock);
        children.insert(NodeType::CMarkNodeParagraph);
        children.insert(NodeType::CMarkNodeHeading);
        children.insert(NodeType::CMarkNodeThematicBreak);
        children.insert(NodeType::CMarkNodeText);
        children.insert(NodeType::CMarkNodeSoftbreak);
        children.insert(NodeType::CMarkNodeLinebreak);
        children.insert(NodeType::CMarkNodeCode);
        children.insert(NodeType::CMarkNodeHtmlInline);
        children.insert(NodeType::CMarkNodeCustomInline);
        children.insert(NodeType::CMarkNodeEmph);
        children.insert(NodeType::CMarkNodeStrong);
        children.insert(NodeType::CMarkNodeLink);
        children.insert(NodeType::CMarkNodeImage);
        children
    };
}

/// Valid child types of Paragraph elements
lazy_static! {
    pub static ref PARAGRAPH_CHILDREN: HashSet<NodeType> = {
        let mut children = HashSet::new();
        children.insert(NodeType::CMarkNodeText);
        children.insert(NodeType::CMarkNodeEmph);
        children.insert(NodeType::CMarkNodeCode);
        children.insert(NodeType::CMarkNodeLink);
        children.insert(NodeType::CMarkNodeImage);
        children.insert(NodeType::CMarkNodeSoftbreak);
        children.insert(NodeType::CMarkNodeLinebreak);
        children.insert(NodeType::CMarkNodeHtmlInline);
        children.insert(NodeType::CMarkNodeCustomInline);
        children.insert(NodeType::CMarkNodeStrong);
        children
    };
}

/// Valid child types of HEADING elements
lazy_static! {
    pub static ref HEADING_CHILDREN: HashSet<NodeType> = { PARAGRAPH_CHILDREN.clone() };
}

/// Valid child types of Thematic Break elements
lazy_static! {
    pub static ref THEMATIC_BREAK_CHILDREN: HashSet<NodeType> = { HashSet::new() };
}

/// Valid child types of Text elements
lazy_static! {
    pub static ref TEXT_CHILDREN: HashSet<NodeType> = { HashSet::new() };
}

/// Valid child types of Soft Break elements
lazy_static! {
    pub static ref SOFT_BREAK_CHILDREN: HashSet<NodeType> = { HashSet::new() };
}

/// Valid child types of Line Break elements
lazy_static! {
    pub static ref LINE_BREAK_CHILDREN: HashSet<NodeType> = { HashSet::new() };
}

/// Valid child types of inline Code elements
lazy_static! {
    pub static ref CODE_CHILDREN: HashSet<NodeType> = { HashSet::new() };
}

/// Valid child types of inline HTML elements
lazy_static! {
    pub static ref INLINE_HTML_CHILDREN: HashSet<NodeType> = { HashSet::new() };
}

/// Valid child types of Custom Inline elements
lazy_static! {
    pub static ref CUSTOM_INLINE_CHILDREN: HashSet<NodeType> = { PARAGRAPH_CHILDREN.clone() };
}

/// Valid child types of Emph elements
lazy_static! {
    pub static ref EMPH_CHILDREN: HashSet<NodeType> = { PARAGRAPH_CHILDREN.clone() };
}

/// Valid child types of Strong elements
lazy_static! {
    pub static ref STRONG_CHILDREN: HashSet<NodeType> = { PARAGRAPH_CHILDREN.clone() };
}

/// Valid child types of Link elements
lazy_static! {
    pub static ref LINK_CHILDREN: HashSet<NodeType> = { PARAGRAPH_CHILDREN.clone() };
}

/// Valid child types of Image elements
lazy_static! {
    pub static ref IMAGE_CHILDREN: HashSet<NodeType> = { PARAGRAPH_CHILDREN.clone() };
}
