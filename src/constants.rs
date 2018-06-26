extern crate try_from;

use self::try_from::TryFrom;
use super::{DoogieError, DoogieResult};

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

/// Each Node in the libcmark document AST possesses a type attribute that corresponds to it's
/// equivalent CommonMark semantic element.
#[derive(PartialEq, Debug, Clone)]
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
pub const DOCUMENT_CHILDREN: &[NodeType] = &[
    NodeType::CMarkNodeParagraph,
    NodeType::CMarkNodeHeading,
    NodeType::CMarkNodeThematicBreak,
    NodeType::CMarkNodeCodeBlock,
    NodeType::CMarkNodeHtmlBlock,
    NodeType::CMarkNodeCustomBlock,
    NodeType::CMarkNodeList,
    NodeType::CMarkNodeBlockQuote
];

/// Valid child types of List elements
pub const LIST_CHILDREN: &[NodeType] = &[
    NodeType::CMarkNodeItem
];

/// Valid child types of List Item elements
pub const ITEM_CHILDREN: &[NodeType] = DOCUMENT_CHILDREN;

/// Valid child types of Block Quote elements
pub const BLOCK_QUOTE_CHILDREN: &[NodeType] = DOCUMENT_CHILDREN;

/// Valid child types of Code Block elements
pub const CODE_BLOCK_CHILDREN: &[NodeType] = &[];

/// Valid child types of HTML Block elements
pub const HTML_BLOCK_CHILDREN: &[NodeType] = &[];

/// Valid child types of Custom Block elements
pub const CUSTOM_BLOCK_CHILDREN: &[NodeType] = &[
    NodeType::CMarkNodeBlockQuote,
    NodeType::CMarkNodeList,
    NodeType::CMarkNodeItem,
    NodeType::CMarkNodeCodeBlock,
    NodeType::CMarkNodeHtmlBlock,
    NodeType::CMarkNodeCustomBlock,
    NodeType::CMarkNodeParagraph,
    NodeType::CMarkNodeHeading,
    NodeType::CMarkNodeThematicBreak ,
    NodeType::CMarkNodeText ,
    NodeType::CMarkNodeSoftbreak ,
    NodeType::CMarkNodeLinebreak ,
    NodeType::CMarkNodeCode ,
    NodeType::CMarkNodeHtmlInline ,
    NodeType::CMarkNodeCustomInline ,
    NodeType::CMarkNodeEmph ,
    NodeType::CMarkNodeStrong ,
    NodeType::CMarkNodeLink ,
    NodeType::CMarkNodeImage ,
];

/// Valid child types of Paragraph elements
pub const PARAGRAPH_CHILDREN: &[NodeType] = &[
    NodeType::CMarkNodeText,
    NodeType::CMarkNodeEmph,
    NodeType::CMarkNodeCode,
    NodeType::CMarkNodeLink,
    NodeType::CMarkNodeImage,
    NodeType::CMarkNodeSoftbreak,
    NodeType::CMarkNodeLinebreak,
    NodeType::CMarkNodeHtmlInline,
    NodeType::CMarkNodeCustomInline,
    NodeType::CMarkNodeStrong,
];

/// Valid child types of HEADING elements
pub const HEADING_CHILDREN: &[NodeType] = PARAGRAPH_CHILDREN;

/// Valid child types of Thematic Break elements
pub const THEMATIC_BREAK_CHILDREN: &[NodeType] = &[];

/// Valid child types of Text elements
pub const TEXT_CHILDREN: &[NodeType] = &[];

/// Valid child types of Soft Break elements
pub const SOFT_BREAK_CHILDREN: &[NodeType] = &[];

/// Valid child types of Line Break elements
pub const LINE_BREAK_CHILDREN: &[NodeType] = &[];

/// Valid child types of inline Code elements
pub const CODE_CHILDREN: &[NodeType] = &[];

/// Valid child types of inline HTML elements
pub const INLINE_HTML_CHILDREN: &[NodeType] = &[];

/// Valid child types of Custom Inline elements
pub const CUSTOM_INLINE_CHILDREN: &[NodeType] = PARAGRAPH_CHILDREN;

/// Valid child types of Emph elements
pub const EMPH_CHILDREN: &[NodeType] = PARAGRAPH_CHILDREN;

/// Valid child types of Strong elements
pub const STRONG_CHILDREN: &[NodeType] = PARAGRAPH_CHILDREN;

/// Valid child types of Link elements
pub const LINK_CHILDREN: &[NodeType] = PARAGRAPH_CHILDREN;

/// Valid child types of Image elements
pub const IMAGE_CHILDREN: &[NodeType] = PARAGRAPH_CHILDREN;