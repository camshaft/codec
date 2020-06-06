use crate::buffer::{Result, SliceableBuffer};

pub trait Decoder<T, B: SliceableBuffer>: Sized {
    fn decode_from(self, buffer: B) -> Result<T, B>;
}

pub trait TypeDecoder<B: SliceableBuffer>: Sized {
    fn decode_type(buffer: B) -> Result<Self, B>;
}
