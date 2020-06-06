use crate::{
    buffer::{
        BorrowedBuffer, BorrowedMutBuffer, FiniteBuffer, FiniteMutBuffer, Result, SliceableBuffer,
        SliceableMutBuffer,
    },
    decode::TypeDecoder,
};

macro_rules! impl_slice {
    ($a:lifetime, $ty:ty, $split:ident) => {
        impl<$a> SliceableBuffer for $ty {
            type Slice = $ty;

            #[inline(always)]
            fn slice(
                self,
                offset: usize,
            ) -> Result<Self::Slice, Self> {
                let ((), buffer) = self.ensure_len(offset)?;
                let (a, b) = buffer.$split(offset);
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
            #[inline(always)]
            fn decode_type(buffer: B) -> Result<Self, B> {
                let (slice, rest) = buffer.consume();
                Ok((slice.into(), rest))
            }
        }

        impl<$a> BorrowedBuffer<$a> for $ty {
            #[inline(always)]
            fn into_less_safe_slice(self) -> &$a [u8] {
                &self[..]
            }
        }
    };
}

impl_slice!('a, &'a [u8], split_at);
impl_slice!('a, &'a mut [u8], split_at_mut);

impl FiniteMutBuffer for &mut [u8] {
    #[inline(always)]
    fn as_less_safe_mut_slice(&mut self) -> &mut [u8] {
        self
    }
}

impl<'a> SliceableMutBuffer for &'a mut [u8] {
    type FrozenSlice = &'a [u8];

    #[inline(always)]
    fn freeze(self) -> Self::FrozenSlice {
        self
    }
}

impl<'a> BorrowedMutBuffer<'a> for &'a mut [u8] {
    #[inline(always)]
    fn into_less_safe_mut_slice(self) -> &'a mut [u8] {
        self
    }
}
