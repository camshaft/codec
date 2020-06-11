use crate::{
    buffer::{
        BorrowedBuffer, BorrowedMutBuffer, BufferError, FiniteBuffer, FiniteMutBuffer, Result,
        SplittableBuffer, SplittableMutBuffer,
    },
    decode::TypeDecoder,
    encode::{EncoderBuffer, TypeEncoder},
};

macro_rules! impl_slice {
    ($a:lifetime, $ty:ty, $split:ident) => {
        impl<$a> SplittableBuffer for $ty {
            type Slice = $ty;

            #[inline(always)]
            fn checked_split(
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

impl<'a> SplittableMutBuffer for &'a mut [u8] {
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
    fn encoder_capacity(&self) -> usize {
        self.len()
    }

    #[inline(always)]
    fn encode_bytes<T: AsRef<[u8]>>(self, bytes: T) -> Result<usize, Self> {
        let bytes = bytes.as_ref();
        let len = bytes.len();
        let (mut slice, buffer) = self.checked_split(len)?;
        slice.as_less_safe_mut_slice().copy_from_slice(bytes);
        Ok((len, buffer))
    }

    #[inline(always)]
    fn checkpoint<F>(self, f: F) -> Result<usize, Self>
    where
        F: FnOnce(Self) -> Result<(), Self>,
    {
        let buffer_len = self.len();

        unsafe {
            use core::slice::from_raw_parts_mut;
            let view = from_raw_parts_mut(self.as_mut_ptr(), buffer_len);

            match f(view) {
                Ok(((), buffer)) => {
                    let consumed_len = buffer_len - buffer.encoder_capacity();
                    let _ = buffer;
                    let (_, buffer) = self.split_at_mut(consumed_len);
                    Ok((consumed_len, buffer))
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

#[cfg(test)]
mod tests {
    encoder_buffer_tests!(
        &mut [u8],
        |len, out| {
            let mut buffer = [0; len];
            out = &mut buffer[..];
        },
        |_final| { &buffer[..] }
    );
}

#[cfg(test)]
mod double_borrow_tests {
    encoder_buffer_tests!(
        &mut [u8],
        |len, out| {
            let mut buffer = [0; len];
            let borrowed = &mut buffer[..];
            out = &mut borrowed[..];
        },
        |_final| { borrowed }
    );
}
