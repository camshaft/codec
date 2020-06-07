use crate::{
    buffer::{BufferError, BufferErrorReason, Result},
    encode::{EncoderBuffer, TypeEncoder},
};

pub type LenResult = core::result::Result<usize, BufferErrorReason>;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct LenEstimator {
    start: usize,
    end: usize,
}

impl LenEstimator {
    #[inline(always)]
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

    #[inline(always)]
    fn capacity(&self) -> usize {
        self.end - self.start
    }

    #[inline(always)]
    fn encode_bytes<T: AsRef<[u8]>>(self, bytes: T) -> Result<usize, Self> {
        let len = bytes.as_ref().len();
        let (_, mut buffer) = self.ensure_capacity(len)?;
        buffer.start += len;
        Ok((len, buffer))
    }

    #[inline(always)]
    fn encode_checkpoint<F>(self, f: F) -> Result<Self::Slice, Self>
    where
        F: FnOnce(Self) -> Result<(), Self>,
    {
        let mut prev = self;
        match f(self) {
            Ok(((), next)) => {
                prev.end = next.start;
                Ok((prev, next))
            }
            Err(err) => Err(BufferError {
                reason: err.reason,
                buffer: prev,
            }),
        }
    }
}
