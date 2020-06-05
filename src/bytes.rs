use crate::decode::{DecoderError, FiniteBuffer, PeekBuffer, SliceableBuffer, SliceableMutBuffer};
pub use bytes::{Bytes, BytesMut};

macro_rules! impl_bytes {
    ($name:ident) => {
        impl SliceableBuffer for $name {
            type Slice = $name;

            fn slice(mut self, offset: usize) -> std::result::Result<(Self, Self), DecoderError> {
                self.ensure_len(offset)?;
                let b = self.split_off(offset);
                Ok((self, b))
            }

            fn slice_with<T, E, F: FnOnce(PeekBuffer) -> Result<T, E>>(
                mut self,
                len: usize,
                f: F,
            ) -> Result<(T, Self), E>
            where
                E: From<DecoderError>,
            {
                self.ensure_len(len)?;
                let value = f(PeekBuffer::new(&self[..len]))?;
                drop(self.split_to(len));
                Ok((value, self))
            }
        }

        impl FiniteBuffer for $name {
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

    fn freeze(self) -> Self::FrozenSlice {
        BytesMut::freeze(self)
    }
}
