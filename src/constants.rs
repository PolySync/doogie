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

/// Each Node in the document tree possesses a type attribute that corresponds to it's equivalent
/// CommonMark semantic element.
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

/// List Nodes have one of these types associated with them.
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

/// Ordered List items have a delimiter attribute.
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
