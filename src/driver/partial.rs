use std::{fs, io};
use std::num::ParseIntError;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use thiserror::Error;
use crate::driver::{seal, State, Driver, Full, BlockStatus};
use crate::{Kind, OnceCell};
use crate::base::HardBlockReasons;

#[cfg_attr(feature = "extra-traits", derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash))]
pub enum Partial {}

impl State for Partial {}

impl seal::StateSeal for Partial {
    type Repr = seal::PartiallyReadRepr;
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("io error")]
    Io(#[from] #[source] io::Error),

    #[error("parse error")]
    Parse(#[from] #[source] parse_display::ParseError),

    #[error("parse int error")]
    ParseInt(#[from] #[source] ParseIntError),

    #[error("invalid numeric value for bool (expected 0 or 1, found '{value}')")]
    InvalidNumericValueForBool { value: String },
}

impl From<PathBuf> for Driver<Partial> {
    fn from(path: PathBuf) -> Self {
        Self::from_path(path)
    }
}

impl Driver<Partial> {
    pub fn from_path(path: PathBuf) -> Self {
        Self(seal::PartiallyReadRepr {
            path,
            ..Default::default()
        })
    }
}

impl Driver<Partial> {
    fn try_read<'a, T, E, JW, P>(&'a self, field: &'a OnceCell<T>, join_with: JW, parser: P) -> Result<&T, E>
        where
            E: From<io::Error>,
            JW: AsRef<Path>,
            P: FnOnce(String) -> Result<T, E>,
    {
        field.get_or_try_init(|| parser(fs::read_to_string(self.0.path.join(join_with))?))
    }

    fn try_read_bool<'a>(&'a self, field: &'a OnceCell<bool>, join_with: impl AsRef<Path>) -> Result<bool> {
        self.try_read(field, join_with, |value| match value.as_str() {
            "0" => Ok(false),
            "1" => Ok(true),
            _ => Err(Error::InvalidNumericValueForBool { value })
        }).copied()
    }

    fn try_read_from_str<'a, T, E, JW>(&'a self, field: &'a OnceCell<T>, join_with: JW) -> Result<&T, E>
    where
        T: FromStr,
        E: From<io::Error> + From<T::Err>,
        JW: AsRef<Path>,
    {
        self.try_read(field, join_with, |v| T::from_str(&v).map_err(Into::into))
    }

    pub fn try_name(&self) -> io::Result<&str> {
        self.try_read(&self.0.name, "name", Ok).map(Deref::deref)
    }

    pub fn try_kind(&self) -> Result<Kind> {
        self.try_read_from_str(&self.0.kind, "type").copied()
    }

    pub fn try_persistent(&self) -> Result<bool> {
        self.try_read_bool(&self.0.persistent, "persistent")
    }

    pub fn try_soft_blocked(&self) -> Result<bool> {
        self.try_read_bool(&self.0.soft_blocked, "soft")
    }

    pub fn try_hard_blocked(&self) -> Result<bool> {
        self.try_read_bool(&self.0.hard_blocked, "hard")
    }

    pub fn try_hard_block_reasons_unchecked(&self) -> Result<HardBlockReasons> {
        self.try_read_from_str(&self.0.hard_block_reasons, "hard_block_reasons").copied()
    }

    pub fn try_hard_block_reasons(&self) -> Result<Option<HardBlockReasons>> {
        if self.try_hard_blocked()? {
            self.try_hard_block_reasons_unchecked().map(Some)
        } else {
            Ok(None)
        }
    }

    pub fn try_block_status(&self) -> Result<BlockStatus> {
        Ok(BlockStatus::from_soft_and_hard(self.try_soft_blocked()?, self.try_hard_block_reasons()?))
    }

    fn fully_read_impl(&mut self) -> Result<Driver<Full>> {
        Ok(Driver(seal::FullyReadRepr {
            name: {
                let _ = self.try_name()?;
                self.0.name.take().unwrap()
            },
            kind: self.try_kind()?,
            persistent: self.try_persistent()?,
            block_status: self.try_block_status()?,
        }))
    }

    pub fn fully_read(mut self) -> Result<Driver<Full>, (Error, Box<Self>)> {
        self.fully_read_impl().map_err(|e| (e, Box::new(self)))
    }
}

