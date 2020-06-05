use super::{Decoder, DecoderError, PeekBuffer, PeekMutBuffer, TypeDecoder};

pub trait SliceableBuffer: Sized {
    type Slice: FiniteBuffer;

    fn slice(self, len: usize) -> Result<(Self::Slice, Self), DecoderError>;

    #[inline(always)]
    fn slice_with<T, E, F: FnOnce(PeekBuffer) -> Result<T, E>>(
        self,
        len: usize,
        f: F,
    ) -> Result<(T, Self), E>
    where
        E: From<DecoderError>,
    {
        let (a, b) = self.slice(len)?;
        let v = f(PeekBuffer::new(a.as_less_safe_slice()))?;
        Ok((v, b))
    }

    #[inline(always)]
    fn decode<T: TypeDecoder<Self>>(self) -> Result<(T, Self), T::Error> {
        T::decode_from(self)
    }

    #[inline(always)]
    fn decode_with<T, D: Decoder<T, Self>>(self, decoder: D) -> Result<(T, Self), D::Error> {
        decoder.decode_from(self)
    }
}

pub trait SliceableMutBuffer: SliceableBuffer {
    type FrozenSlice: SliceableBuffer;

    fn freeze(self) -> Self::FrozenSlice;
}

pub trait FiniteBuffer: SliceableBuffer {
    fn as_less_safe_slice(&self) -> &[u8];

    fn peek(&self) -> PeekBuffer {
        PeekBuffer::new(self.as_less_safe_slice())
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
        self.slice(len).unwrap()
    }

    #[inline(always)]
    fn ensure_len(&self, expected: usize) -> Result<(), DecoderError> {
        let actual = self.len();
        if actual >= expected {
            Ok(())
        } else {
            Err(DecoderError::UnexpectedEof { actual, expected })
        }
    }

    #[inline(always)]
    fn ensure_empty(&self) -> Result<(), DecoderError> {
        if self.is_empty() {
            Ok(())
        } else {
            Err(DecoderError::UnexpectedBytes { len: self.len() })
        }
    }
}

pub trait FiniteMutBuffer: FiniteBuffer {
    fn as_less_safe_slice_mut(&mut self) -> &mut [u8];

    fn peek_mut(&mut self) -> PeekMutBuffer {
        PeekMutBuffer::new(self.as_less_safe_slice_mut())
    }
}
