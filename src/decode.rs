use crate::buffer::{Result, SplittableBuffer};

pub trait Decoder<T, B: SplittableBuffer>: Sized {
    fn decode_from(self, buffer: B) -> Result<T, B>;
}

pub trait TypeDecoder<B: SplittableBuffer>: Sized {
    fn decode_type(buffer: B) -> Result<Self, B>;
}
