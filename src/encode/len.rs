use crate::{
    buffer::{
        BufferError, BufferErrorReason, FiniteBuffer, FiniteMutBuffer, Result, SplittableBuffer,
        SplittableMutBuffer,
    },
    encode::{EncoderBuffer, TypeEncoder},
};

pub type LenResult = core::result::Result<usize, BufferErrorReason>;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct LenEstimator {
    len: usize,
}

impl LenEstimator {
    #[inline(always)]
    pub fn encoding_len<T>(value: T, capacity: usize) -> LenResult
    where
        T: TypeEncoder<Self>,
    {
        let estimator = LenEstimator { len: capacity };
        match estimator.encode(value) {
            Ok((len, _estimator)) => Ok(len),
            Err(err) => Err(err.reason),
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl EncoderBuffer for LenEstimator {
    #[inline(always)]
    fn encoder_capacity(&self) -> usize {
        self.len()
    }

    #[inline(always)]
    fn encode_bytes<T: AsRef<[u8]>>(self, bytes: T) -> Result<usize, Self> {
        let len = bytes.as_ref().len();
        let (_, buffer) = self.checked_split(len)?;
        Ok((len, buffer))
    }

    #[inline(always)]
    fn checkpoint<F>(self, f: F) -> Result<usize, Self>
    where
        F: FnOnce(Self) -> Result<(), Self>,
    {
        let prev = self;
        match f(self) {
            Ok(((), next)) => {
                let consumed_len = prev.len() - next.len();
                Ok((consumed_len, next))
            }
            Err(err) => Err(BufferError {
                reason: err.reason,
                buffer: prev,
            }),
        }
    }
}

impl FiniteBuffer for LenEstimator {
    fn as_less_safe_slice(&self) -> &[u8] {
        panic!("cannot read the slice of a len estimator");
    }

    fn len(&self) -> usize {
        Self::len(self)
    }
}

impl FiniteMutBuffer for LenEstimator {
    fn as_less_safe_mut_slice(&mut self) -> &mut [u8] {
        panic!("cannot read the mut slice of a len estimator");
    }
}

impl SplittableBuffer for LenEstimator {
    type Slice = Self;

    fn checked_split(self, len: usize) -> Result<Self::Slice, Self> {
        let (_, buffer) = self.ensure_encoder_capacity(len)?;
        let slice = LenEstimator { len };
        let buffer = LenEstimator {
            len: buffer.len - len,
        };
        Ok((slice, buffer))
    }
}

impl SplittableMutBuffer for LenEstimator {
    type FrozenSlice = Self;

    fn freeze(self) -> Self::FrozenSlice {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    encoder_buffer_tests!(LenEstimator, |len, out| {
        out = LenEstimator { len };
    });
}
