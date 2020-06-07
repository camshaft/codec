use crate::{
    buffer::{
        BorrowedBuffer, BorrowedMutBuffer, BufferError, FiniteBuffer, FiniteMutBuffer, Result,
        SliceableBuffer, SliceableMutBuffer,
    },
    decode::TypeDecoder,
    encode::{EncodeWithCursor, EncoderBuffer, EncoderCursor, TypeEncoder},
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

    #[inline(always)]
    fn capacity(&self) -> usize {
        self.len()
    }

    #[inline(always)]
    fn encode_bytes<T: AsRef<[u8]>>(self, bytes: T) -> Result<usize, Self> {
        let bytes = bytes.as_ref();
        let len = bytes.len();
        let (mut slice, buffer) = self.slice(len)?;
        slice.as_less_safe_mut_slice().copy_from_slice(bytes);
        Ok((len, buffer))
    }

    #[inline(always)]
    fn encode_checkpoint<F>(self, f: F) -> Result<Self::Slice, Self>
    where
        F: FnOnce(Self) -> Result<(), Self>,
    {
        let buffer_len = self.len();

        let view = unsafe {
            use core::slice::from_raw_parts_mut;
            from_raw_parts_mut(self.as_mut_ptr(), buffer_len)
        };

        match f(view) {
            Ok(((), buffer)) => {
                let consumed_len = buffer_len - buffer.capacity();
                drop(buffer);
                Ok(self.split_at_mut(consumed_len))
            }
            Err(err) => {
                let reason = err.reason;
                drop(err);
                Err(BufferError {
                    reason,
                    buffer: self,
                })
            }
        }
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

impl<B: EncoderBuffer, T> TypeEncoder<B> for &[T]
where
    for<'a> &'a T: TypeEncoder<B>,
{
    #[inline(always)]
    fn encode_type(self, mut buffer: B) -> Result<(), B> {
        for item in self {
            let (_, next) = buffer.encode(item)?;
            buffer = next;
        }
        Ok(((), buffer))
    }
}

impl<B: EncoderBuffer, T> TypeEncoder<B> for &&[T]
where
    for<'a> &'a T: TypeEncoder<B>,
{
    #[inline(always)]
    fn encode_type(self, mut buffer: B) -> Result<(), B> {
        for item in self.iter() {
            let (_, next) = buffer.encode(item)?;
            buffer = next;
        }
        Ok(((), buffer))
    }
}
