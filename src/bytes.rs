use crate::{
    buffer::{
        FiniteBuffer, FiniteMutBuffer, LookaheadBuffer, LookaheadMutBuffer, Result,
        SplittableBuffer, SplittableMutBuffer,
    },
    encode::EncoderBuffer,
};
use bytes::BufMut;
pub use bytes::{Bytes, BytesMut};

macro_rules! impl_bytes {
    ($name:ident) => {
        impl SplittableBuffer for $name {
            type Slice = $name;

            #[inline(always)]
            fn checked_split(self, offset: usize) -> Result<Self::Slice, Self> {
                let (_, mut buffer) = self.ensure_len(offset)?;
                let b = buffer.split_off(offset);
                Ok((buffer, b))
            }

            #[inline(always)]
            fn checked_split_with<T, F: FnOnce(LookaheadBuffer) -> Result<T, LookaheadBuffer>>(
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

impl SplittableMutBuffer for BytesMut {
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

macro_rules! impl_encoder_buffer {
    ($ty:ty) => {
        impl EncoderBuffer for $ty {
            #[inline(always)]
            fn encoder_capacity(&self) -> usize {
                BufMut::remaining_mut(self)
            }

            #[inline(always)]
            fn encode_bytes<T: AsRef<[u8]>>(self, bytes: T) -> Result<usize, Self> {
                let bytes = bytes.as_ref();
                let len = bytes.len();
                #[allow(unused_mut)]
                let (_, mut buffer) = self.ensure_encoder_capacity(len)?;
                buffer.extend_from_slice(bytes);
                Ok((len, buffer))
            }

            #[inline(always)]
            fn checkpoint<F>(self, f: F) -> Result<usize, Self>
            where
                F: FnOnce(Self) -> Result<(), Self>,
            {
                let initial_len = self.len();

                match f(self) {
                    Ok(((), buffer)) => Ok((buffer.len() - initial_len, buffer)),
                    #[allow(unused_mut)]
                    Err(mut err) => {
                        // roll back the len to the initial value
                        unsafe { err.buffer.set_len(initial_len) };
                        Err(err)
                    }
                }
            }
        }
    };
}

impl_encoder_buffer!(BytesMut);
impl_encoder_buffer!(&mut BytesMut);

// TODO specialize on bytes for zero copy
macro_rules! impl_codec {
    ($ty:ty, | $slice:ident | $new:expr) => {
        impl<B: FiniteBuffer> crate::decode::TypeDecoder<B> for $ty {
            #[inline(always)]
            fn decode_type(buffer: B) -> Result<$ty, B> {
                let (slice, buffer) = buffer.consume();
                let $slice = slice.as_less_safe_slice();
                let value = $new;
                Ok((value, buffer))
            }
        }

        impl<B: EncoderBuffer> crate::encode::TypeEncoder<B> for $ty {
            #[inline(always)]
            fn encode_type(self, buffer: B) -> Result<(), B> {
                let (_, buffer) = buffer.encode_bytes(&self[..])?;
                Ok(((), buffer))
            }
        }

        impl<B: EncoderBuffer> crate::encode::TypeEncoder<B> for &$ty {
            #[inline(always)]
            fn encode_type(self, buffer: B) -> Result<(), B> {
                let (_, buffer) = buffer.encode_bytes(&self[..])?;
                Ok(((), buffer))
            }
        }
    };
}

impl_codec!(Bytes, |slice| Bytes::copy_from_slice(slice));
impl_codec!(BytesMut, |slice| {
    let mut buffer = BytesMut::with_capacity(slice.len());
    buffer.extend_from_slice(slice);
    buffer
});

#[cfg(test)]
mod tests {
    use super::BytesMut;

    mod bytes_owned {
        use super::*;

        encoder_buffer_tests!(
            BytesMut,
            |len, out| {
                out = BytesMut::with_capacity(len);
            },
            |buffer| {
                buffer.resize(len.max(buffer.len()), 0);
                &buffer[..]
            }
        );
    }

    mod bytes_ref {
        use super::*;
        encoder_buffer_tests!(
            &mut BytesMut,
            |len, out| {
                let mut buffer = BytesMut::with_capacity(len);
                out = &mut buffer;
            },
            |buffer| {
                buffer.resize(len.max(buffer.len()), 0);
                &buffer[..]
            }
        );
    }
}
