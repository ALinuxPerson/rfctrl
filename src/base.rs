use std::io::{Read, Write};
use std::{io, mem};
use std::num::ParseIntError;
use std::str::FromStr;
use bitflags::bitflags;
use parse_display::{Display, FromStr};

pub const CHAR_DEVICE: &str = "/dev/rfkill";

pub trait ReadFrom: Sized {
    type Error;

    fn read_from<R: Read>(reader: R) -> Result<Self, Self::Error>;
}

impl ReadFrom for u8 {
    type Error = io::Error;

    fn read_from<R: Read>(mut reader: R) -> Result<Self, Self::Error> {
        let mut buf = [0];
        reader.read_exact(&mut buf)?;
        Ok(buf[0])
    }
}

impl ReadFrom for bool {
    type Error = io::Error;

    fn read_from<R: Read>(reader: R) -> Result<Self, Self::Error> {
        Ok(u8::read_from(reader)? != 0)
    }
}

impl ReadFrom for u32 {
    type Error = io::Error;

    fn read_from<R: Read>(mut reader: R) -> Result<Self, Self::Error> {
        let mut buf = [0; 4];
        reader.read_exact(&mut buf)?;
        Ok(u32::from_ne_bytes(buf))
    }
}

pub trait WriteTo {
    type Error;

    fn write_to<W: Write>(&self, writer: W) -> Result<(), Self::Error>;
}

#[derive(Copy, Clone, Display, FromStr)]
#[cfg_attr(feature = "extra-traits", derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash))]
#[repr(u8)]
pub enum Kind {
    #[display("all")]
    All = 0,

    #[display("wifi")]
    WirelessLan,

    #[display("bluetooth")]
    Bluetooth,

    #[display("uwb")]
    UltraWideBand,

    #[display("wimax")]
    WiMax,

    #[display("wwan")]
    WirelessWan,

    #[display("gps")]
    Gps,

    #[display("fm")]
    Fm,

    #[display("nfc")]
    Nfc,
}

impl Kind {
    pub const fn from_u8(value: u8) -> Option<Self> {
        use Kind::*;

        let value = match value {
            0 => All,
            1 => WirelessLan,
            2 => Bluetooth,
            3 => UltraWideBand,
            4 => WiMax,
            5 => WirelessWan,
            6 => Gps,
            7 => Fm,
            8 => Nfc,
            _ => return None,
        };

        Some(value)
    }
}

impl ReadFrom for Kind {
    type Error = io::Error;

    fn read_from<R: Read>(reader: R) -> Result<Self, Self::Error> {
        let byte = u8::read_from(reader)?;
        Self::from_u8(byte).ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, format!("invalid discriminant {byte} for Kind")))
    }
}

#[cfg_attr(feature = "extra-traits", derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash))]
pub enum Operation {
    Add,
    Delete,
    Change { all: bool },
}

impl Operation {
    pub const fn from_u8(value: u8) -> Option<Self> {
        use Operation::*;

        let value = match value {
            0 => Add,
            1 => Delete,
            2 => Change { all: false },
            3 => Change { all: true },
            _ => return None,
        };

        Some(value)
    }

    pub const fn to_u8(&self) -> u8 {
        match self {
            Self::Add => 0,
            Self::Delete => 1,
            Self::Change { all: false } => 2,
            Self::Change { all: true } => 3,
        }
    }
}

impl ReadFrom for Operation {
    type Error = io::Error;

    fn read_from<R: Read>(reader: R) -> Result<Self, Self::Error> {
        let byte = u8::read_from(reader)?;
        Self::from_u8(byte).ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, format!("invalid discriminant {byte} for Operation")))
    }
}

#[derive(Copy, Clone)]
#[cfg_attr(feature = "extra-traits", derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash))]
pub struct Block {
    pub soft: bool,
    pub(crate) hard: Option<HardBlockReasons>,
}

impl Block {
    pub(crate) const fn new(soft: bool, hard: Option<HardBlockReasons>) -> Self {
        Self { soft, hard }
    }

    pub const fn hard(&self) -> bool {
        self.hard.is_some()
    }

    pub const fn hard_block_reasons(&self) -> Option<HardBlockReasons> {
        self.hard
    }
}

impl ReadFrom for Block {
    type Error = io::Error;

    fn read_from<R: Read>(mut reader: R) -> Result<Self, Self::Error> {
        let soft = bool::read_from(&mut reader)?;
        let hard = bool::read_from(&mut reader)?;
        let hard = if hard {
            Some(HardBlockReasons::read_from(reader)?)
        } else {
            None
        };

        Ok(Self::new(soft, hard))
    }
}

#[cfg_attr(feature = "extra-traits", derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash))]
pub struct Event {
    pub index: usize,
    pub kind: Kind,
    pub operation: Operation,
    pub block: Block,
}

impl Event {
    pub const SIZE: usize = mem::size_of::<u32>() // idx
        + mem::size_of::<u8>() // type
        + mem::size_of::<u8>() // op
        + mem::size_of::<u8>() // soft
        + mem::size_of::<u8>() // hard
        + mem::size_of::<u8>(); // hard_block_reasons (if hard is true)
}

impl ReadFrom for Event {
    type Error = io::Error;

    fn read_from<R: Read>(mut reader: R) -> Result<Self, Self::Error> {
        let index = u32::read_from(&mut reader)? as usize;
        let kind = Kind::read_from(&mut reader)?;
        let operation = Operation::read_from(&mut reader)?;
        let block = Block::read_from(reader)?;
        Ok(Self { index, kind, operation, block })
    }
}

bitflags! {
    pub struct HardBlockReasons: u8 {
        const SIGNAL = 1 << 0;
        const NOT_OWNER = 1 << 1;
    }
}

impl ReadFrom for HardBlockReasons {
    type Error = io::Error;

    fn read_from<R: Read>(reader: R) -> Result<Self, Self::Error> {
        Ok(HardBlockReasons::from_bits_truncate(u8::read_from(reader)?))
    }
}

impl FromStr for HardBlockReasons {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bits = u8::from_str_radix(s.trim_start_matches("0x"), 16)?;
        Ok(Self::from_bits_truncate(bits))
    }
}
