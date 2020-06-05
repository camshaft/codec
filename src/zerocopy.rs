use crate::decode::{
    Decoder, DecoderError, FiniteBuffer, FiniteMutBuffer, SliceableBuffer, SliceableMutBuffer,
    TypeDecoder,
};
use core::{
    marker::PhantomData,
    mem::size_of,
    ops::{Deref, DerefMut},
};
use zerocopy::{AsBytes, FromBytes, LayoutVerified, Unaligned};

#[derive(FromBytes, Unaligned, Copy, Clone, Debug)]
#[repr(C)]
pub struct ZerocopyBuffer<Buffer>(Buffer);

impl<T: FromBytes + Unaligned, Buffer: FiniteBuffer> AsRef<T> for ZerocopyBuffer<Buffer> {
    fn as_ref(&self) -> &T {
        <LayoutVerified<&[u8], T>>::new_unaligned(self.0.as_less_safe_slice())
            .unwrap()
            .into_ref()
    }
}

impl<T: AsBytes + FromBytes + Unaligned, Buffer: FiniteMutBuffer> AsMut<T>
    for ZerocopyBuffer<Buffer>
{
    fn as_mut(&mut self) -> &mut T {
        <LayoutVerified<&mut [u8], T>>::new_unaligned(self.0.as_less_safe_slice_mut())
            .unwrap()
            .into_mut()
    }
}

#[derive(FromBytes, Unaligned, Clone, Copy, Debug)]
#[repr(C)]
pub struct Ref<T, Buffer> {
    buffer: ZerocopyBuffer<Buffer>,
    value: PhantomData<T>,
}

impl<T, Buffer> Deref for Ref<T, Buffer>
where
    ZerocopyBuffer<Buffer>: AsRef<T>,
{
    type Target = T;

    fn deref(&self) -> &T {
        self.buffer.as_ref()
    }
}

impl<T: FromBytes + Unaligned, B: SliceableBuffer> TypeDecoder<B> for Ref<T, B::Slice> {
    type Error = DecoderError;

    fn decode_from(buffer: B) -> Result<(Self, B), Self::Error> {
        let (owner, buffer) = buffer.slice(size_of::<T>())?;
        let value = Ref {
            buffer: ZerocopyBuffer(owner),
            value: PhantomData,
        };
        Ok((value, buffer))
    }
}

#[derive(FromBytes, Unaligned, Clone, Copy, Debug)]
#[repr(C)]
pub struct Mut<T, Buffer> {
    buffer: ZerocopyBuffer<Buffer>,
    value: PhantomData<T>,
}

impl<T, Buffer> Deref for Mut<T, Buffer>
where
    ZerocopyBuffer<Buffer>: AsRef<T>,
{
    type Target = T;

    fn deref(&self) -> &T {
        self.buffer.as_ref()
    }
}

impl<T, Buffer> DerefMut for Mut<T, Buffer>
where
    ZerocopyBuffer<Buffer>: AsRef<T> + AsMut<T>,
{
    fn deref_mut(&mut self) -> &mut T {
        self.buffer.as_mut()
    }
}

impl<T: AsBytes + FromBytes + Unaligned, B: SliceableMutBuffer> TypeDecoder<B>
    for Mut<T, B::Slice>
{
    type Error = DecoderError;

    fn decode_from(buffer: B) -> Result<(Self, B), Self::Error> {
        let (owner, buffer) = buffer.slice(size_of::<T>())?;
        let value = Mut {
            buffer: ZerocopyBuffer(owner),
            value: PhantomData,
        };
        Ok((value, buffer))
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Derefed;

impl<T: Copy + FromBytes + Unaligned, B: SliceableBuffer> Decoder<T, B> for Derefed {
    type Error = DecoderError;

    fn decode_from(self, buffer: B) -> Result<(T, B), Self::Error> {
        let (value, buffer) = <Ref<T, B::Slice>>::decode_from(buffer)?;
        Ok((*value, buffer))
    }
}
