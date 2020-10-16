use crate::decode::{Decoder, DecoderBuffer, TypeDecoder};
use core::fmt;

macro_rules! map_buffer_error {
    ($expr:expr, $prev:expr) => {{
        let res = match $expr {
            Ok((value, _)) => Ok(value),
            Err(err) => Err(err.reason),
        };

        match res {
            Ok(value) => (value, $prev),
            Err(reason) => {
                return Err($crate::buffer::BufferError {
                    buffer: $prev,
                    reason,
                });
            }
        }
    }};
}

mod lookahead;

pub use lookahead::*;

#[derive(Clone, Copy, Debug)]
pub struct BufferError<B> {
    pub buffer: B,
    pub reason: BufferErrorReason,
}

impl<B> BufferError<B> {
    pub fn with_buffer<NewB>(self, buffer: NewB) -> BufferError<NewB> {
        BufferError {
            buffer,
            reason: self.reason,
        }
    }

    pub fn map_buffer<NewB, F: Fn(B) -> NewB>(self, map: F) -> BufferError<NewB> {
        BufferError {
            buffer: map(self.buffer),
            reason: self.reason,
        }
    }
}

impl<B> fmt::Display for BufferError<B> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.reason.fmt(f)
    }
}

#[cfg(feature = "std")]
impl<B: core::fmt::Debug> std::error::Error for BufferError<B> {}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BufferErrorReason {
    UnexpectedEof { actual: usize, expected: usize },
    UnexpectedBytes { len: usize },
    InvalidValue { message: &'static str },
}

impl fmt::Display for BufferErrorReason {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::UnexpectedEof { actual, expected } => writeln!(
                f,
                "unexpected EOF: actual = {}, expected = {}",
                actual, expected
            ),
            Self::UnexpectedBytes { len } => writeln!(f, "unexpected bytes: remaining = {}", len),
            Self::InvalidValue { message } => writeln!(f, "invalid value: {}", message),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for BufferErrorReason {}

pub type Result<T, B> = core::result::Result<(T, B), BufferError<B>>;

pub trait SplittableBuffer: Sized {
    type Slice: FiniteBuffer;

    fn checked_split(self, len: usize) -> Result<Self::Slice, Self>;

    #[inline(always)]
    fn checked_split_with<T, F: FnOnce(LookaheadBuffer) -> Result<T, LookaheadBuffer>>(
        self,
        len: usize,
        f: F,
    ) -> Result<T, Self> {
        let (a, b) = self.checked_split(len)?;
        let res = f(LookaheadBuffer::new(a.as_less_safe_slice()));
        let (v, b) = map_buffer_error!(res, b);
        Ok((v, b))
    }
}

pub trait SplittableMutBuffer: SplittableBuffer
where
    Self::Slice: FiniteMutBuffer,
{
    type FrozenSlice: SplittableBuffer;

    #[inline(always)]
    fn slice_mut_with<T, F: FnOnce(LookaheadMutBuffer) -> Result<T, LookaheadMutBuffer>>(
        self,
        len: usize,
        f: F,
    ) -> Result<T, Self> {
        let (mut a, b) = self.checked_split(len)?;
        let res = f(LookaheadMutBuffer::new(a.as_less_safe_mut_slice()));
        let (v, b) = map_buffer_error!(res, b);
        Ok((v, b))
    }

    fn encode_slice(self, bytes: &[u8]) -> Result<Self::Slice, Self> {
        let (mut target, buffer) = self.checked_split(bytes.len())?;

        target.as_less_safe_mut_slice().copy_from_slice(bytes);

        Ok((target, buffer))
    }

    fn freeze(self) -> Self::FrozenSlice;
}

pub trait FiniteBuffer: SplittableBuffer<Slice = Self> {
    fn as_less_safe_slice(&self) -> &[u8];

    #[inline(always)]
    fn peek<'a, T: TypeDecoder<LookaheadBuffer<'a>>>(&'a self) -> Result<T, LookaheadBuffer<'a>> {
        self.lookahead().decode()
    }

    #[inline(always)]
    fn peek_with<'a, T, D: Decoder<T, LookaheadBuffer<'a>>>(
        &'a self,
        decoder: D,
    ) -> Result<T, LookaheadBuffer<'a>> {
        let buffer = self.lookahead();
        decoder.decode_from(buffer)
    }

    #[inline(always)]
    fn lookahead(&self) -> LookaheadBuffer {
        LookaheadBuffer::new(self.as_less_safe_slice())
    }

    #[inline(always)]
    fn len(&self) -> usize {
        self.as_less_safe_slice().len()
    }

    #[inline(always)]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline(always)]
    fn consume(self) -> (Self::Slice, Self) {
        let len = self.len();
        match self.checked_split(len) {
            Ok((slice, buffer)) => (slice, buffer),
            _ => panic!("len was misreported"),
        }
    }

    #[inline(always)]
    fn consumed_decode<T: TypeDecoder<Self>>(self) -> Result<T, Self> {
        let (value, buffer) = T::decode_type(self)?;
        let ((), buffer) = buffer.ensure_empty()?;
        Ok((value, buffer))
    }

    #[inline(always)]
    fn ensure_len(self, expected: usize) -> Result<usize, Self> {
        let actual = self.len();
        if actual >= expected {
            Ok((actual, self))
        } else {
            Err(BufferError {
                reason: BufferErrorReason::UnexpectedEof { actual, expected },
                buffer: self,
            })
        }
    }

    #[inline(always)]
    fn ensure_empty(self) -> Result<(), Self> {
        if self.is_empty() {
            Ok(((), self))
        } else {
            Err(BufferError {
                reason: BufferErrorReason::UnexpectedBytes { len: self.len() },
                buffer: self,
            })
        }
    }
}

pub trait FiniteMutBuffer: FiniteBuffer {
    fn as_less_safe_mut_slice(&mut self) -> &mut [u8];

    #[inline(always)]
    fn lookahead_mut(&mut self) -> LookaheadMutBuffer {
        LookaheadMutBuffer::new(self.as_less_safe_mut_slice())
    }
}

pub trait BorrowedBuffer<'a>: FiniteBuffer {
    fn into_less_safe_slice(self) -> &'a [u8];
}

pub trait BorrowedMutBuffer<'a>: FiniteMutBuffer {
    fn into_less_safe_mut_slice(self) -> &'a mut [u8];
}

// #[cfg(test)]
// mod tests {
//     use crate::{
//         buffer::{FiniteBuffer, SplittableBuffer},
//         bytes::*,
//     };

//     fn peek_check<D: FiniteBuffer>(d: D) {
//         {
//             let t = d.peek_buffer();
//             let (_, t) = t.decode::<u8>().unwrap();
//             let (_, t) = t.decode::<u8>().unwrap();
//             let _ = t;
//         }
//         decode_check(d)
//     }

//     fn decode_check<S: SplittableBuffer>(s: S) {
//         // let (_, s) = s.decode::<u8>().unwrap();
//         // let (_, s) = s.decode::<u8>().unwrap();
//         let _ = s;
//     }

//     #[test]
//     fn slice_test() {
//         let mut i = [1u8, 2, 3];
//         peek_check(&i[..]);
//         peek_check(&mut i[..]);
//         peek_check(Bytes::from(vec![1, 2, 3]));
//     }
// }
