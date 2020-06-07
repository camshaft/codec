use crate::{
    buffer::Result,
    encode::{Encoder, EncoderBuffer, TypeEncoder},
};
use core::marker::PhantomData;

pub trait EncodeWithCursor<'a: 'child, 'child>: EncoderBuffer + 'a {
    type Child: EncoderBuffer;

    #[inline(always)]
    fn cursor_encode<T>(self, value: T) -> Result<EncoderCursor<T, Self::Slice>, Self>
    where
        T: TypeEncoder<<Self as EncodeWithCursor<'a, 'child>>::Child>,
        <Self as EncodeWithCursor<'a, 'child>>::Child: EncoderBuffer,
    {
        self.encode_with_cursor::<T, _>(|slice| value.encode_type(slice))
    }

    #[inline(always)]
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

    #[inline(always)]
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
    #[inline(always)]
    pub fn new(buffer: B) -> Self {
        Self {
            value: PhantomData,
            buffer,
        }
    }
}
