use crate::{
    buffer::{FiniteBuffer, FiniteMutBuffer, Result, SplittableBuffer, SplittableMutBuffer},
    decode::{Decoder, TypeDecoder},
    encode::{EncoderBuffer, TypeEncoder},
};
use core::{cmp::Ordering, marker::PhantomData, mem::size_of, ops};
use zerocopy::{AsBytes, ByteSlice, FromBytes, Unaligned};

#[derive(Copy, Clone, FromBytes, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[doc(hidden)]
#[repr(C)]
pub struct VerifiedBuffer<B>(B);

impl<B: FiniteBuffer> VerifiedBuffer<B> {
    #[inline(always)]
    unsafe fn as_ref<T>(&self) -> &T {
        &*(self.0.as_less_safe_slice().as_ptr() as *const T)
    }
}

impl<B: FiniteMutBuffer> VerifiedBuffer<B> {
    #[inline(always)]
    unsafe fn as_mut<T>(&mut self) -> &mut T {
        &mut *(self.0.as_less_safe_mut_slice().as_ptr() as *mut T)
    }
}

macro_rules! impl_ref {
    ($name:ident, $buffer:ident, $slice_ty:ident, [$($constraint:ident),*] $(, $ensure_alignment:ident)?) => {
        #[derive(FromBytes, Clone, Copy, Debug, Hash)]
        #[repr(C)]
        pub struct $name<T, Buffer> {
            buffer: VerifiedBuffer<Buffer>,
            value: PhantomData<T>,
        }

        impl<T, Buffer: FiniteBuffer> $name<T, Buffer> {
            pub fn as_bytes(&self) -> &[u8] {
                self.buffer.0.as_less_safe_slice()
            }
        }

        impl<T, Buffer: FiniteMutBuffer> $name<T, Buffer> {
            pub fn as_mut_bytes(&mut self) -> &mut [u8] {
                self.buffer.0.as_less_safe_mut_slice()
            }
        }

        impl<T: FromBytes + PartialEq<U> $(+ $constraint)*, B: FiniteBuffer, U> PartialEq<U> for $name<T, B> {
            #[inline(always)]
            fn eq(&self, rhs: &U) -> bool {
                unsafe { self.buffer.as_ref::<T>() }.eq(rhs)
            }
        }

        impl<T: FromBytes + PartialOrd<U> $(+ $constraint)*, B: FiniteBuffer, U> PartialOrd<U> for $name<T, B> {
            #[inline(always)]
            fn partial_cmp(&self, rhs: &U) -> Option<Ordering> {
                unsafe { self.buffer.as_ref::<T>() }.partial_cmp(rhs)
            }
        }

        impl<T: FromBytes $(+ $constraint)*, Buffer: FiniteBuffer> ops::Deref for $name<T, Buffer> {
            type Target = T;

            #[inline(always)]
            fn deref(&self) -> &T {
                unsafe { self.buffer.as_ref() }
            }
        }

        impl<T: FromBytes $(+ $constraint)*, B: $buffer> TypeDecoder<B> for $name<T, B::Slice>
            where B::Slice: $slice_ty,
        {
            #[inline(always)]
            fn decode_type(buffer: B) -> Result<Self, B> {
                let (owner, buffer) = buffer.checked_split(size_of::<T>())?;
                $(
                    {
                        if let Err(err) = owner.lookahead().$ensure_alignment::<T>() {
                            return Err($crate::buffer::BufferError {
                                reason: err.reason,
                                buffer,
                            });
                        }
                    }
                )*
                let value = $name {
                    buffer: VerifiedBuffer(owner),
                    value: PhantomData,
                };
                Ok((value, buffer))
            }
        }

        impl<T, B, E> TypeEncoder<E> for $name<T, B>
        where
            T: AsBytes $(+ $constraint)*,
            B: $slice_ty,
            E: EncoderBuffer,
        {
            #[inline(always)]
            fn encode_type(self, buffer: E) -> Result<(), E> {
                let (_, buffer) = buffer.encode_bytes(self.as_bytes())?;
                Ok(((), buffer))
            }
        }

        impl<T, B, E> TypeEncoder<E> for &$name<T, B>
        where
            T: AsBytes $(+ $constraint)*,
            B: $slice_ty,
            E: EncoderBuffer,
        {
            #[inline(always)]
            fn encode_type(self, buffer: E) -> Result<(), E> {
                let (_, buffer) = buffer.encode_bytes(self.as_bytes())?;
                Ok(((), buffer))
            }
        }
    };
}

impl_ref!(Ref, SplittableBuffer, FiniteBuffer, []);
impl_ref!(Mut, SplittableMutBuffer, FiniteMutBuffer, [AsBytes]);

impl<T: AsBytes + FromBytes + Unaligned, Buffer: FiniteMutBuffer> ops::DerefMut for Mut<T, Buffer> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut T {
        unsafe { self.buffer.as_mut() }
    }
}

macro_rules! impl_deref {
    ($name:ident, $delegate:ident, [$($constraint:ident)*]) => {
        #[derive(Clone, Copy, Debug)]
        pub struct $name;

        impl<T: Copy + FromBytes $(+ $constraint)*, B: FiniteBuffer> Decoder<T, B> for $name {
            #[inline(always)]
            fn decode_from(self, buffer: B) -> Result<T, B> {
                let (value, buffer) = $delegate::decode_type(buffer)?;
                Ok((*value, buffer))
            }
        }
    };
}

impl_deref!(Deref, Ref, [Unaligned]);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decode::DecoderBuffer;

    #[test]
    fn decode_unaligned_ref_test() {
        let buffer = &[0, 1, 2, 3][..];
        let (value, _buffer) = buffer.decode().unwrap();
        let value: Ref<[u8; 2], _> = value;
        assert_eq!(value, [0, 1]);
    }

    #[test]
    fn decode_unaligned_mut_test() {
        let buffer = &mut [0, 1, 2, 3][..];
        let (value, _buffer) = buffer.decode().unwrap();
        let value: Mut<[u8; 2], _> = value;
        assert_eq!(value, [0, 1]);
    }

    #[test]
    fn decode_unaligned_deref_test() {
        let buffer = &[0, 1, 2, 3][..];
        let (value, _buffer) = buffer.decode_with(Deref).unwrap();
        let value: [u8; 2] = value;
        assert_eq!(value, [0, 1]);
    }
}
