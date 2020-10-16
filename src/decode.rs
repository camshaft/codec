use crate::buffer::{FiniteBuffer, Result, SplittableBuffer};

pub trait Decoder<T, B: SplittableBuffer>: Sized {
    fn decode_from(self, buffer: B) -> Result<T, B>;
}

pub trait TypeDecoder<B: SplittableBuffer>: Sized {
    fn decode_type(buffer: B) -> Result<Self, B>;
}

pub trait DecoderBuffer: SplittableBuffer {
    #[inline(always)]
    fn decode<T: TypeDecoder<Self>>(self) -> Result<T, Self> {
        T::decode_type(self)
    }

    #[inline(always)]
    fn decode_with<T, D: Decoder<T, Self>>(self, decoder: D) -> Result<T, Self> {
        decoder.decode_from(self)
    }
}

impl<B: SplittableBuffer> DecoderBuffer for B {}

#[derive(Clone, Copy, Debug, Default)]
pub struct Skip(pub usize);

impl<B: FiniteBuffer> TypeDecoder<B> for Skip {
    fn decode_type(buffer: B) -> Result<Self, B> {
        let (slice, buffer) = buffer.consume();
        let len = slice.len();
        Ok((Self(len), buffer))
    }
}
