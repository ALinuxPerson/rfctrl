use crate::{Driver, Kind};
use crate::driver::{BlockStatus, seal, State};

#[cfg_attr(feature = "extra-traits", derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash))]
pub enum Full {}

impl State for Full {}

impl seal::StateSeal for Full {
    type Repr = seal::FullyReadRepr;
}

impl Driver<Full> {
    pub fn name(&self) -> &str {
        &self.0.name
    }

    pub const fn kind(&self) -> Kind {
        self.0.kind
    }

    pub const fn persistent(&self) -> bool {
        self.0.persistent
    }

    pub const fn block_status(&self) -> BlockStatus {
        self.0.block_status
    }
}
