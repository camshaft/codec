use crate::buffer::{BufferError, BufferErrorReason, Result};
use core::mem::size_of;

mod cursor;
mod len;

pub use cursor::*;
pub use len::*;

pub trait Encoder<T, B>: Sized
where
    B: EncoderBuffer,
{
    fn encode_into(self, value: T, buffer: B) -> Result<(), B>;
}

pub trait TypeEncoder<B>: Sized
where
    B: EncoderBuffer,
{
    fn encode_type(self, buffer: B) -> Result<(), B>;
}

impl<'a, B: EncoderBuffer, T> TypeEncoder<B> for &'a &T
where
    &'a T: TypeEncoder<B>,
{
    #[inline(always)]
    fn encode_type(self, buffer: B) -> Result<(), B> {
        (*self).encode_type(buffer)
    }
}

impl<'a, B: EncoderBuffer, T> TypeEncoder<B> for &'a mut &mut T
where
    &'a mut T: TypeEncoder<B>,
{
    #[inline(always)]
    fn encode_type(self, buffer: B) -> Result<(), B> {
        (*self).encode_type(buffer)
    }
}

impl<'a, B: EncoderBuffer, T> TypeEncoder<B> for &'a mut &T
where
    &'a T: TypeEncoder<B>,
{
    #[inline(always)]
    fn encode_type(self, buffer: B) -> Result<(), B> {
        (*self).encode_type(buffer)
    }
}

impl<'a, B: EncoderBuffer, T> TypeEncoder<B> for &'a &mut T
where
    &'a T: TypeEncoder<B>,
{
    #[inline(always)]
    fn encode_type(self, buffer: B) -> Result<(), B> {
        (**self).encode_type(buffer)
    }
}

pub trait EncoderBuffer: Sized {
    type Slice: EncoderBuffer;

    fn capacity(&self) -> usize;

    #[inline(always)]
    fn encode<T>(self, value: T) -> Result<usize, Self>
    where
        T: TypeEncoder<Self>,
    {
        let (slice, buffer) = self.encode_checkpoint(|buffer| value.encode_type(buffer))?;
        Ok((slice.capacity(), buffer))
    }

    #[inline(always)]
    fn encode_with<T, E>(self, value: T, encoder: E) -> Result<usize, Self>
    where
        E: Encoder<T, Self>,
    {
        let (slice, buffer) =
            self.encode_checkpoint(|buffer| encoder.encode_into(value, buffer))?;
        Ok((slice.capacity(), buffer))
    }

    #[inline(always)]
    fn try_encode<T>(self, value: T) -> (core::result::Result<usize, BufferErrorReason>, Self)
    where
        T: TypeEncoder<Self>,
    {
        match self.encode_checkpoint(|buffer| value.encode_type(buffer)) {
            Ok((slice, buffer)) => (Ok(slice.capacity()), buffer),
            Err(err) => (Err(err.reason), err.buffer),
        }
    }

    #[inline(always)]
    fn try_encode_with<T, E>(
        self,
        value: T,
        encoder: E,
    ) -> (core::result::Result<usize, BufferErrorReason>, Self)
    where
        E: Encoder<T, Self>,
    {
        match self.encode_checkpoint(|buffer| encoder.encode_into(value, buffer)) {
            Ok((slice, buffer)) => (Ok(slice.capacity()), buffer),
            Err(err) => (Err(err.reason), err.buffer),
        }
    }

    #[inline(always)]
    fn encoding_len<T>(&self, value: T) -> LenResult
    where
        T: TypeEncoder<LenEstimator>,
    {
        LenEstimator::encoding_len(value, self.capacity())
    }

    fn encode_bytes<T: AsRef<[u8]>>(self, bytes: T) -> Result<usize, Self>;

    #[inline(always)]
    fn encode_repeated<T>(self, value: T, count: usize) -> Result<usize, Self>
    where
        T: Copy + TypeEncoder<Self>,
    {
        let len = size_of::<T>() * count;
        let (_, buffer) = self.ensure_capacity(len)?;

        let (_, buffer) = buffer.encode_checkpoint(|mut buffer| {
            for _ in 0..count {
                let (_, b) = buffer.encode(value)?;
                buffer = b;
            }

            Ok(((), buffer))
        })?;

        Ok((len, buffer))
    }

    #[inline(always)]
    fn encode_repeated_ref<'a, T>(self, value: &'a T, count: usize) -> Result<usize, Self>
    where
        &'a T: TypeEncoder<Self>,
    {
        let (slice, buffer) = self.encode_checkpoint(|mut buffer| {
            for _ in 0..count {
                let (_, b) = buffer.encode(value)?;
                buffer = b;
            }

            Ok(((), buffer))
        })?;

        Ok((slice.capacity(), buffer))
    }

    #[inline(always)]
    fn ensure_capacity(self, expected: usize) -> Result<usize, Self> {
        let actual = self.capacity();
        if actual >= expected {
            Ok((actual, self))
        } else {
            Err(BufferError {
                reason: BufferErrorReason::UnexpectedEof { actual, expected },
                buffer: self,
            })
        }
    }

    fn encode_checkpoint<F>(self, f: F) -> Result<Self::Slice, Self>
    where
        F: FnOnce(Self) -> Result<(), Self>;
}
