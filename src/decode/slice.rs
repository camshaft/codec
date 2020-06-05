use crate::decode::{
    DecoderError, FiniteBuffer, FiniteMutBuffer, SliceableBuffer, SliceableMutBuffer, TypeDecoder,
};

macro_rules! impl_slice {
    ($a:lifetime, $ty:ty, $split:ident) => {
        impl<$a> SliceableBuffer for $ty {
            type Slice = $ty;

            #[inline(always)]
            fn slice(
                self,
                offset: usize,
            ) -> std::result::Result<(Self::Slice, Self), DecoderError> {
                self.ensure_len(offset)?;
                let (a, b) = self.$split(offset);
                Ok((a, b))
            }
        }

        impl<$a> FiniteBuffer for $ty {
            #[inline(always)]
            fn as_less_safe_slice(&self) -> &[u8] {
                self.as_ref()
            }
        }

        impl<$a, B: FiniteBuffer> TypeDecoder<B> for $ty
        where
            B::Slice: Into<$ty>,
        {
            type Error = DecoderError;

            #[inline(always)]
            fn decode_from(buffer: B) -> Result<(Self, B), Self::Error> {
                let (slice, rest) = buffer.consume();
                Ok((slice.into(), rest))
            }
        }
    };
}

impl_slice!('a, &'a [u8], split_at);
impl_slice!('a, &'a mut [u8], split_at_mut);

impl FiniteMutBuffer for &mut [u8] {
    #[inline(always)]
    fn as_less_safe_slice_mut(&mut self) -> &mut [u8] {
        self.as_mut()
    }
}

impl<'a> SliceableMutBuffer for &'a mut [u8] {
    type FrozenSlice = &'a [u8];

    #[inline(always)]
    fn freeze(self) -> Self::FrozenSlice {
        self
    }
}
