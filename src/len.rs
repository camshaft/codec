use crate::{
    buffer::{
        BufferError, FiniteBuffer, FiniteMutBuffer, Result, SliceableBuffer, SliceableMutBuffer,
    },
    decode::{Decoder, TypeDecoder},
    encode::{Encoder, EncoderBuffer, LenEstimator, TypeEncoder},
};
use core::{
    convert::{TryFrom, TryInto},
    marker::PhantomData,
};
use num_traits::bounds::Bounded;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LenPrefixed<T, L> {
    pub len: L,
    pub value: T,
}

impl<T, L, B> TypeDecoder<B> for LenPrefixed<T, L>
where
    B: FiniteBuffer,
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

impl<T, L, E> Encoder<T, E> for LenPrefix<L>
where
    E: EncoderBuffer + SliceableBuffer + SliceableMutBuffer,
    <E as SliceableBuffer>::Slice: EncoderBuffer + FiniteMutBuffer,
    L: TypeEncoder<<E as SliceableBuffer>::Slice>
        + TypeEncoder<LenEstimator>
        + TryInto<usize>
        + TryFrom<usize>
        + Bounded
        + Copy,
    T: TypeEncoder<<E as SliceableBuffer>::Slice>,
    for<'a> &'a T: TypeEncoder<LenEstimator>,
{
    #[inline(always)]
    fn encode_into(self, value: T, buffer: E) -> Result<(), E> {
        let capacity = buffer.capacity();

        // compute the maximum prefix given the current buffer capacity
        let max_value: L = capacity.try_into().unwrap_or(L::max_value());

        // compute the maximum bytes the prefix needs
        let prefix_len = match LenEstimator::encoding_len(max_value, capacity) {
            Ok(len) => len,
            Err(reason) => return Err(BufferError { reason, buffer }),
        };

        // remove the prefix requirement
        let capacity = capacity - prefix_len;

        // bind the capacity to the maximum encodable prefix
        let capacity = capacity.min(L::max_value().try_into().unwrap_or(usize::MAX));

        // compute how many bytes the value needs
        let value_len = match LenEstimator::encoding_len(&value, capacity) {
            Ok(len) => len,
            Err(reason) => return Err(BufferError { reason, buffer }),
        };

        // compute the actual prefix
        let prefix: L = value_len.try_into().ok().expect("len should always fit");

        // compute how many bytes the actual prefix needs
        let prefix_len = match LenEstimator::encoding_len(prefix, capacity) {
            Ok(len) => len,
            Err(reason) => return Err(BufferError { reason, buffer }),
        };

        // slice off the buffer to ensure the value encoder has the correct capacity
        let (slice, buffer) = <E as SliceableBuffer>::slice(buffer, prefix_len + value_len)?;

        // perform the actual encoding
        match Ok(((), slice))
            .and_then(|(_, slice)| slice.encode(prefix))
            .and_then(|(_, slice)| slice.encode(value))
            .and_then(|(_, slice)| slice.ensure_empty())
        {
            Ok((_, _)) => Ok(((), buffer)),
            Err(err) => Err(err.with_buffer(buffer)),
        }
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
    fn encode_len_prefix_test() {
        let mut buffer = [0; 3];
        let slice = &mut buffer[..];
        let (_, _) = slice.encode_with(123u16, LenPrefix::new::<u8>()).unwrap();
        let (_, _) = slice.encode_with(&456u16, LenPrefix::new::<u8>()).unwrap();
        let (_, _) = slice
            .encode_with(&mut 789u16, LenPrefix::new::<u8>())
            .unwrap();
    }

    #[test]
    #[should_panic]
    fn encode_len_prefix_encoding_cap_test() {
        let mut buffer = [0; 512];
        let slice = &mut buffer[..];
        let value = [1u8; 256];
        let (_, _) = slice
            .encode_with(&value[..], LenPrefix::new::<u8>())
            .unwrap();
    }

    #[test]
    #[should_panic]
    fn encode_len_prefix_cap_test() {
        let mut buffer = [0; 4];
        let slice = &mut buffer[..];
        let value = [1u8; 16];
        let (_, _) = slice
            .encode_with(&value[..], LenPrefix::new::<u8>())
            .unwrap();
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
