use crate::decode::{Decoder, SliceableBuffer};

macro_rules! impl_endian {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
        pub struct $name;

        impl<T, B: SliceableBuffer> Decoder<T, B> for &$name
        where
            $name: Decoder<T, B>,
        {
            type Error = <$name as Decoder<T, B>>::Error;

            #[inline(always)]
            fn decode_from(self, buffer: B) -> Result<(T, B), Self::Error> {
                $name.decode_from(buffer)
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

impl<T, B: SliceableBuffer, Error> Decoder<T, B> for Endian
where
    Big: Decoder<T, B, Error = Error>,
    Little: Decoder<T, B, Error = Error>,
{
    type Error = Error;

    #[inline(always)]
    fn decode_from(self, buffer: B) -> Result<(T, B), Self::Error> {
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
    type Error = <Endian as Decoder<T, B>>::Error;

    #[inline(always)]
    fn decode_from(self, buffer: B) -> Result<(T, B), Self::Error> {
        (*self).decode_from(buffer)
    }
}

pub const NETWORK: Big = Big;

#[cfg(target_endian = "little")]
pub const NATIVE: Little = Little;

#[cfg(target_endian = "big")]
pub const NATIVE: Big = Big;
