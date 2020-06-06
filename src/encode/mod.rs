use crate::buffer::{BufferError, BufferErrorReason, Result};
use core::marker::PhantomData;

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

pub trait EncoderBuffer: Sized {
    type Slice;

    fn capacity(&self) -> usize;

    fn encode<T>(self, value: T) -> Result<usize, Self>
    where
        T: TypeEncoder<Self>,
    {
        let initial_capacity = self.capacity();
        let (_, buffer) = value.encode_type(self)?;
        let len = initial_capacity - buffer.capacity();
        Ok((len, buffer))
    }

    fn encoding_len<T>(&self, value: T) -> LenResult
    where
        T: TypeEncoder<LenEstimator>,
    {
        LenEstimator::encoding_len(value, self.capacity())
    }

    fn encode_with<T, E>(self, value: T, encoder: E) -> Result<usize, Self>
    where
        E: Encoder<T, Self>,
    {
        let initial_capacity = self.capacity();
        let (_, buffer) = encoder.encode_into(value, self)?;
        let len = initial_capacity - buffer.capacity();
        Ok((len, buffer))
    }

    fn encode_bytes(self, bytes: &[u8]) -> Result<usize, Self>;

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
}

pub trait EncodeWithCursor<'a: 'child, 'child>: EncoderBuffer + 'a {
    type Child: EncoderBuffer;

    fn cursor_encode<T>(self, value: T) -> Result<EncoderCursor<T, Self::Slice>, Self>
    where
        T: TypeEncoder<<Self as EncodeWithCursor<'a, 'child>>::Child>,
        <Self as EncodeWithCursor<'a, 'child>>::Child: EncoderBuffer,
    {
        self.encode_with_cursor::<T, _>(|slice| value.encode_type(slice))
    }

    fn cursor_encode_with<T, E>(
        self,
        value: T,
        encoder: E,
    ) -> Result<EncoderCursor<T, Self::Slice>, Self>
    where
        E: Encoder<T, <Self as EncodeWithCursor<'a, 'child>>::Child>,
        <Self as EncodeWithCursor<'a, 'child>>::Child: EncoderBuffer,
    {
        self.encode_with_cursor::<T, _>(|slice| encoder.encode_into(value, slice))
    }

    fn cursor_encode_bytes(self, bytes: &[u8]) -> Result<EncoderCursor<&[u8], Self::Slice>, Self>
    where
        <Self as EncodeWithCursor<'a, 'child>>::Child: EncoderBuffer,
    {
        self.encode_with_cursor::<&[u8], _>(|slice| {
            let (_, slice) = slice.encode_bytes(bytes)?;
            Ok(((), slice))
        })
    }

    fn encode_with_cursor<T, F>(self, f: F) -> Result<EncoderCursor<T, Self::Slice>, Self>
    where
        F: FnOnce(Self::Child) -> Result<(), Self::Child>;
}

pub struct EncoderCursor<T, B> {
    value: PhantomData<T>,
    buffer: B,
}

impl<T, B> EncoderCursor<T, B> {
    pub fn new(buffer: B) -> Self {
        Self {
            value: PhantomData,
            buffer,
        }
    }
}

pub type LenResult = core::result::Result<usize, BufferErrorReason>;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct LenEstimator {
    start: usize,
    end: usize,
}

impl LenEstimator {
    pub fn encoding_len<T>(value: T, capacity: usize) -> LenResult
    where
        T: TypeEncoder<Self>,
    {
        let estimator = LenEstimator {
            start: 0,
            end: capacity,
        };
        match estimator.encode(value) {
            Ok((len, _)) => Ok(len),
            Err(err) => Err(err.reason),
        }
    }
}

impl EncoderBuffer for LenEstimator {
    type Slice = Self;

    fn capacity(&self) -> usize {
        self.end - self.start
    }

    fn encode_bytes(self, bytes: &[u8]) -> Result<usize, Self> {
        let len = bytes.len();
        let (_, mut buffer) = self.ensure_capacity(len)?;
        buffer.start += len;
        Ok((len, buffer))
    }
}
