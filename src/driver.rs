mod seal;
pub mod partial;
pub mod full;
pub mod event_based {
    use crate::driver::{seal, State};
    pub(crate) use seal::EventBasedRepr;

    #[cfg_attr(feature = "extra-traits", derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash))]
    pub enum EventBased {}

    impl State for EventBased {}

    impl seal::StateSeal for EventBased {
        type Repr = EventBasedRepr;
    }
}

pub use event_based::EventBased;
pub use partial::Partial;
pub use full::Full;
use crate::{Block, HardBlockReasons};

pub trait State: seal::StateSeal {}

#[cfg_attr(feature = "extra-traits", derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash))]
pub struct Driver<S: State>(pub(crate) S::Repr);

#[derive(Copy, Clone)]
#[cfg_attr(feature = "extra-traits", derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash))]
pub enum BlockStatusRepr {
    Blocked(Block),
    Unblocked,
}

#[derive(Copy, Clone)]
#[cfg_attr(feature = "extra-traits", derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash))]
pub struct BlockStatus(BlockStatusRepr);

impl BlockStatus {
    pub const fn from_soft_and_hard(soft_block: bool, hard_block: Option<HardBlockReasons>) -> Self {
        Self::from_block(Block { soft: soft_block, hard: hard_block })
    }

    pub const fn from_block(block: Block) -> Self {
        use BlockStatusRepr::*;

        match block {
            Block { soft: false, hard: None } => Self(Unblocked),
            _ => Self(Blocked(block)),
        }
    }

    pub const fn inner(&self) -> BlockStatusRepr {
        self.0
    }

    pub const fn get(&self) -> (bool, Option<HardBlockReasons>) {
        use BlockStatusRepr::*;

        match self.0 {
            Blocked(Block { soft, hard }) => (soft, hard),
            Unblocked => (false, None),
        }
    }

    pub const fn soft_blocked(&self) -> bool {
        let (soft, _) = self.get();
        soft
    }

    pub const fn hard_block_reasons(&self) -> Option<HardBlockReasons> {
        let (_, hard) = self.get();
        hard
    }

    pub const fn hard_blocked(&self) -> bool {
        let (_, hard) = self.get();
        hard.is_some()
    }

    pub const fn blocked(&self) -> bool {
        let (soft, hard) = self.get();
        soft || hard.is_some()
    }
}

