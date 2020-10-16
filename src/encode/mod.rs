use crate::buffer::{BufferError, BufferErrorReason, Result};
use core::mem::size_of;

#[cfg(test)]
#[macro_use]
mod test_macros;

// mod cursor;
mod len;

// pub use cursor::*;
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
    fn encoder_capacity(&self) -> usize;

    #[inline(always)]
    fn encode<T>(self, value: T) -> Result<usize, Self>
    where
        T: TypeEncoder<Self>,
    {
        self.checkpoint(|buffer| value.encode_type(buffer))
    }

    #[inline(always)]
    fn encode_with<T, E>(self, value: T, encoder: E) -> Result<usize, Self>
    where
        E: Encoder<T, Self>,
    {
        self.checkpoint(|buffer| encoder.encode_into(value, buffer))
    }

    #[inline(always)]
    fn try_encode<T>(self, value: T) -> (core::result::Result<usize, BufferErrorReason>, Self)
    where
        T: TypeEncoder<Self>,
    {
        match self.checkpoint(|buffer| value.encode_type(buffer)) {
            Ok((len, buffer)) => (Ok(len), buffer),
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
        match self.checkpoint(|buffer| encoder.encode_into(value, buffer)) {
            Ok((len, buffer)) => (Ok(len), buffer),
            Err(err) => (Err(err.reason), err.buffer),
        }
    }

    #[inline(always)]
    fn encoding_len<T>(&self, value: T) -> LenResult
    where
        T: TypeEncoder<LenEstimator>,
    {
        LenEstimator::encoding_len(value, self.encoder_capacity())
    }

    fn encode_bytes<T: AsRef<[u8]>>(self, bytes: T) -> Result<usize, Self>;

    #[inline(always)]
    fn encode_repeated<T>(self, value: T, count: usize) -> Result<usize, Self>
    where
        T: Copy + TypeEncoder<Self>,
    {
        let len = size_of::<T>() * count;
        let (_, buffer) = self.ensure_encoder_capacity(len)?;

        let (_, buffer) = buffer.checkpoint(|mut buffer| {
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
        self.checkpoint(|mut buffer| {
            for _ in 0..count {
                let (_, b) = buffer.encode(value)?;
                buffer = b;
            }

            Ok(((), buffer))
        })
    }

    #[inline(always)]
    fn ensure_encoder_capacity(self, expected: usize) -> Result<usize, Self> {
        let actual = self.encoder_capacity();
        if actual >= expected {
            Ok((actual, self))
        } else {
            Err(BufferError {
                reason: BufferErrorReason::UnexpectedEof { actual, expected },
                buffer: self,
            })
        }
    }

    fn checkpoint<F>(self, f: F) -> Result<usize, Self>
    where
        F: FnOnce(Self) -> Result<(), Self>;
}
