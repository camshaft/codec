use crate::{
    buffer::{
        BorrowedBuffer, BorrowedMutBuffer, BufferError, FiniteBuffer, FiniteMutBuffer, Result,
        SliceableBuffer, SliceableMutBuffer,
    },
    decode::TypeDecoder,
    encode::{EncodeWithCursor, EncoderBuffer, EncoderCursor},
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
                let (_len, buffer) = self.ensure_len(offset)?;
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

impl EncoderBuffer for &mut [u8] {
    type Slice = Self;

    fn capacity(&self) -> usize {
        self.len()
    }

    fn encode_bytes(self, bytes: &[u8]) -> Result<usize, Self> {
        let len = bytes.len();
        let (mut slice, buffer) = self.slice(len)?;
        slice.as_less_safe_mut_slice().copy_from_slice(bytes);
        Ok((len, buffer))
    }
}

impl<'a: 'child, 'child> EncodeWithCursor<'a, 'child> for &'a mut [u8] {
    type Child = &'child mut [u8];

    fn encode_with_cursor<T, F>(self, f: F) -> Result<EncoderCursor<T, Self::Slice>, Self>
    where
        F: FnOnce(Self::Child) -> Result<(), Self::Child>,
    {
        let initial_capacity = self.capacity();

        let consumed_len = {
            let child = unsafe {
                use core::slice::from_raw_parts_mut;
                let buffer_ptr = self.as_mut_ptr();
                let buffer_len = self.len();
                from_raw_parts_mut(buffer_ptr, buffer_len)
            };
            match f(child) {
                Ok(((), buffer)) => initial_capacity
                    .checked_sub(buffer.capacity())
                    .expect("invalid final buffer len"),
                Err(err) => {
                    return Err(BufferError {
                        reason: err.reason,
                        buffer: self,
                    })
                }
            }
        };

        let (_, buffer) = self.ensure_capacity(consumed_len)?;

        match buffer.slice(consumed_len) {
            Ok((slice, buffer)) => {
                let cursor = EncoderCursor::new(slice);
                Ok((cursor, buffer))
            }
            Err(_) => panic!("misreported len"),
        }
    }
}
