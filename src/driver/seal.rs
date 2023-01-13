use std::path::PathBuf;
use crate::{HardBlockReasons, OnceCell};
use crate::driver::BlockStatus;
use crate::Kind;

#[derive(Default)]
#[cfg_attr(feature = "extra-traits", derive(Debug, Eq, PartialEq, Clone))]
pub struct PartiallyReadRepr {
    pub path: PathBuf,
    pub name: OnceCell<String>,
    pub kind: OnceCell<Kind>,
    pub persistent: OnceCell<bool>,
    pub soft_blocked: OnceCell<bool>,
    pub hard_blocked: OnceCell<bool>,
    pub hard_block_reasons: OnceCell<HardBlockReasons>,
}

#[cfg_attr(feature = "extra-traits", derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash))]
pub struct FullyReadRepr {
    pub name: String,
    pub kind: Kind,
    pub persistent: bool,
    pub block_status: BlockStatus,
}

#[cfg_attr(feature = "extra-traits", derive(Debug, Copy, Clone, Eq, PartialEq))]
pub struct EventBasedRepr {
    pub index: usize,
    pub kind: Kind,
    pub block_status: BlockStatus,
}

pub trait StateSeal {
    type Repr;
}
