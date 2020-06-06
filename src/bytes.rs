use crate::buffer::{
    FiniteBuffer, FiniteMutBuffer, PeekBuffer, PeekMutBuffer, Result, SliceableBuffer,
    SliceableMutBuffer,
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
                let ((), buffer) = self.ensure_len(len)?;
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
        let ((), mut buffer) = self.ensure_len(len)?;
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
