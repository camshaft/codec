use crate::{
    buffer::{Result, SplittableBuffer},
    decode::{Decoder, DecoderBuffer, TypeDecoder},
    encode::{Encoder, EncoderBuffer, TypeEncoder},
    len::LenPrefix,
};

pub struct TLV<T, L, V> {
    pub tag: T,
    pub len: LenPrefix<L>,
    pub value: V,
}

impl<B, T, L, V> TypeDecoder<B> for TLV<T, L, V>
where
    B: SplittableBuffer,
    T: TypeDecoder<B>,
    LenPrefix<L>: Decoder<V, B>,
{
    fn decode_type(buffer: B) -> Result<Self, B> {
        let (tag, buffer) = buffer.decode()?;
        let len = LenPrefix::new::<L>();
        let (value, buffer) = buffer.decode_with(len)?;

        Ok((Self { tag, len, value }, buffer))
    }
}

impl<B, T, L, V> TypeEncoder<B> for TLV<T, L, V>
where
    B: EncoderBuffer,
    T: TypeEncoder<B>,
    LenPrefix<L>: Encoder<V, B>,
{
    fn encode_type(self, buffer: B) -> Result<(), B> {
        let (_, buffer) = buffer.encode(self.tag)?;
        let (_, buffer) = buffer.encode_with(self.value, self.len)?;
        Ok(((), buffer))
    }
}

impl<'a, B, T, L, V> TypeEncoder<B> for &'a TLV<T, L, V>
where
    B: EncoderBuffer,
    &'a T: TypeEncoder<B>,
    LenPrefix<L>: Encoder<&'a V, B>,
{
    fn encode_type(self, buffer: B) -> Result<(), B> {
        let (_, buffer) = buffer.encode(&self.tag)?;
        let (_, buffer) = buffer.encode_with(&self.value, self.len)?;
        Ok(((), buffer))
    }
}
