use crate::{
    buffer::{
        BufferError, FiniteBuffer, FiniteMutBuffer, PeekBuffer, PeekMutBuffer, Result,
        SliceableBuffer, SliceableMutBuffer,
    },
    encode::{EncodeWithCursor, EncoderBuffer, EncoderCursor},
};
pub use bytes::{Bytes, BytesMut};

macro_rules! impl_bytes {
    ($name:ident) => {
        impl SliceableBuffer for $name {
            type Slice = $name;

            #[inline(always)]
            fn slice(self, offset: usize) -> Result<Self::Slice, Self> {
                let (_, mut buffer) = self.ensure_len(offset)?;
                let b = buffer.split_off(offset);
                Ok((buffer, b))
            }

            #[inline(always)]
            fn slice_with<T, F: FnOnce(PeekBuffer) -> Result<T, PeekBuffer>>(
                self,
                len: usize,
                f: F,
            ) -> Result<T, Self> {
                let (_len, buffer) = self.ensure_len(len)?;
                let (value, mut buffer) =
                    map_buffer_error!(f(PeekBuffer::new(&buffer[..len])), buffer);
                drop(buffer.split_to(len));
                Ok((value, buffer))
            }
        }

        impl FiniteBuffer for $name {
            #[inline(always)]
            fn as_less_safe_slice(&self) -> &[u8] {
                self.as_ref()
            }
        }
    };
}

impl_bytes!(Bytes);
impl_bytes!(BytesMut);

impl SliceableMutBuffer for BytesMut {
    type FrozenSlice = Bytes;

    #[inline(always)]
    fn slice_mut_with<T, F: FnOnce(PeekMutBuffer) -> Result<T, PeekMutBuffer>>(
        self,
        len: usize,
        f: F,
    ) -> Result<T, Self> {
        let (_len, mut buffer) = self.ensure_len(len)?;
        let (value, mut buffer) =
            map_buffer_error!(f(PeekMutBuffer::new(&mut buffer[..len])), buffer);
        drop(buffer.split_to(len));
        Ok((value, buffer))
    }

    #[inline(always)]
    fn freeze(self) -> Self::FrozenSlice {
        BytesMut::freeze(self)
    }
}

impl FiniteMutBuffer for BytesMut {
    fn as_less_safe_mut_slice(&mut self) -> &mut [u8] {
        self.as_mut()
    }
}

impl EncoderBuffer for BytesMut {
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

impl<'a: 'child, 'child> EncodeWithCursor<'a, 'child> for BytesMut {
    type Child = &'child mut [u8];

    fn encode_with_cursor<T, F>(mut self, f: F) -> Result<EncoderCursor<T, Self::Slice>, Self>
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
