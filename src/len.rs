use crate::{
    buffer::{FiniteBuffer, Result},
    decode::{Decoder, TypeDecoder},
};
use core::{convert::TryInto, marker::PhantomData};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LenPrefixed<T, L> {
    pub len: L,
    pub value: T,
}

impl<T, L, B: FiniteBuffer> TypeDecoder<B> for LenPrefixed<T, L>
where
    L: TypeDecoder<B> + TryInto<usize> + Copy,
    T: TypeDecoder<B::Slice>,
{
    #[inline(always)]
    fn decode_type(buffer: B) -> Result<Self, B> {
        let (len, buffer) = buffer.decode::<L>()?;
        // If it doesn't fit then we most likely won't be able to read it anyway
        let slice_len = len.try_into().unwrap_or(usize::MAX);
        let (slice, buffer) = buffer.slice(slice_len)?;
        let (value, buffer) = map_buffer_error!(slice.consumed_decode(), buffer);
        Ok((Self { len, value }, buffer))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LenPrefix<L>(PhantomData<L>);

impl<L> Default for LenPrefix<L> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl LenPrefix<()> {
    #[allow(clippy::new_ret_no_self)]
    pub const fn new<L>() -> LenPrefix<L> {
        LenPrefix(PhantomData)
    }
}

impl<L, T, B: FiniteBuffer> Decoder<T, B> for LenPrefix<L>
where
    L: TypeDecoder<B> + TryInto<usize> + Copy,
    T: TypeDecoder<B::Slice>,
{
    #[inline(always)]
    fn decode_from(self, buffer: B) -> Result<T, B> {
        let (value, buffer) = <LenPrefixed<T, L>>::decode_type(buffer)?;
        Ok((value.value, buffer))
    }
}

#[cfg(test)]
mod tests {
    use crate::{buffer::*, len::*};

    #[test]
    fn decode_len_prefix_test() {
        let buffer = &[2, 0, 1][..];
        let (value, _buffer) = buffer.decode_with(LenPrefix::new::<u8>()).unwrap();
        let value: u16 = value;
        assert_eq!(value, 1);
    }

    #[test]
    fn decode_with_len_prefix_test() {
        let buffer = &[2, 0, 1][..];
        let (value, _buffer) = buffer.decode_with_len_prefix::<_, u8>().unwrap();
        let value: u16 = value;
        assert_eq!(value, 1);
    }

    #[test]
    fn decode_value_len_prefix_test() {
        let buffer = &[2, 0, 1][..];
        let (value, _buffer) = buffer.decode::<LenPrefixed<_, u8>>().unwrap();
        let value: u16 = value.value;
        assert_eq!(value, 1);
    }
}
