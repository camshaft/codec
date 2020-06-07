use crate::{
    buffer::{
        BufferError, FiniteBuffer, FiniteMutBuffer, LookaheadBuffer, LookaheadMutBuffer, Result,
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
            fn slice_with<T, F: FnOnce(LookaheadBuffer) -> Result<T, LookaheadBuffer>>(
                self,
                len: usize,
                f: F,
            ) -> Result<T, Self> {
                let (_len, buffer) = self.ensure_len(len)?;
                let (value, mut buffer) =
                    map_buffer_error!(f(LookaheadBuffer::new(&buffer[..len])), buffer);
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
    fn slice_mut_with<T, F: FnOnce(LookaheadMutBuffer) -> Result<T, LookaheadMutBuffer>>(
        self,
        len: usize,
        f: F,
    ) -> Result<T, Self> {
        let (_len, mut buffer) = self.ensure_len(len)?;
        let (value, mut buffer) =
            map_buffer_error!(f(LookaheadMutBuffer::new(&mut buffer[..len])), buffer);
        drop(buffer.split_to(len));
        Ok((value, buffer))
    }

    #[inline(always)]
    fn freeze(self) -> Self::FrozenSlice {
        BytesMut::freeze(self)
    }
}

impl FiniteMutBuffer for BytesMut {
    #[inline(always)]
    fn as_less_safe_mut_slice(&mut self) -> &mut [u8] {
        self.as_mut()
    }
}

impl EncoderBuffer for BytesMut {
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
        let initial_len = self.len();

        match f(self.clone()) {
            Ok(((), buffer)) => {
                let consumed_len = initial_len - buffer.capacity();
                self.slice(consumed_len)
            }
            Err(err) => Err(BufferError {
                reason: err.reason,
                buffer: self,
            }),
        }
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
