use crate::{
    buffer::{Result, SliceableBuffer},
    decode::Decoder,
};

macro_rules! impl_endian {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
        pub struct $name;

        impl<T, B: SliceableBuffer> Decoder<T, B> for &$name
        where
            $name: Decoder<T, B>,
        {
            #[inline(always)]
            fn decode_from(self, buffer: B) -> Result<T, B> {
                $name.decode_from(buffer)
            }
        }

        impl Into<Endian> for $name {
            #[inline(always)]
            fn into(self) -> Endian {
                Endian::$name
            }
        }
    };
}

impl_endian!(Little);
impl_endian!(Big);

#[derive(Clone, Copy, Debug)]
pub enum Endian {
    Big,
    Little,
}

impl<T, B: SliceableBuffer> Decoder<T, B> for Endian
where
    Big: Decoder<T, B>,
    Little: Decoder<T, B>,
{
    #[inline(always)]
    fn decode_from(self, buffer: B) -> Result<T, B> {
        match self {
            Self::Little => Little.decode_from(buffer),
            Self::Big => Big.decode_from(buffer),
        }
    }
}

impl<T, B: SliceableBuffer> Decoder<T, B> for &Endian
where
    Endian: Decoder<T, B>,
{
    #[inline(always)]
    fn decode_from(self, buffer: B) -> Result<T, B> {
        (*self).decode_from(buffer)
    }
}

pub const NETWORK: Big = Big;

#[cfg(target_endian = "little")]
pub const NATIVE: Little = Little;

#[cfg(target_endian = "big")]
pub const NATIVE: Big = Big;
