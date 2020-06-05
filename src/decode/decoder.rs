use crate::{
    decode::{DecoderError, FiniteBuffer, SliceableBuffer},
    endian::{Big, Little, NETWORK},
};
use core::convert::TryInto;

pub trait Decoder<T, B: SliceableBuffer>: Sized {
    type Error;

    fn decode_from(self, buffer: B) -> Result<(T, B), Self::Error>;
}

pub trait TypeDecoder<B: SliceableBuffer>: Sized {
    type Error;

    fn decode_from(buffer: B) -> Result<(Self, B), Self::Error>;
}

macro_rules! impl_byte {
    ($ty:ident) => {
        impl<B: SliceableBuffer> TypeDecoder<B> for $ty {
            type Error = DecoderError;

            #[inline(always)]
            fn decode_from(buffer: B) -> Result<(Self, B), Self::Error> {
                buffer.slice_with(core::mem::size_of::<$ty>(), |slice| {
                    Ok(slice.as_less_safe_slice()[0] as $ty)
                })
            }
        }
    };
}

impl_byte!(u8);
impl_byte!(i8);

macro_rules! impl_integer {
    ($ty:ident) => {
        impl<B: SliceableBuffer> TypeDecoder<B> for $ty {
            type Error = DecoderError;

            #[inline(always)]
            fn decode_from(buffer: B) -> Result<(Self, B), Self::Error> {
                NETWORK.decode_from(buffer)
            }
        }

        impl<B: SliceableBuffer> Decoder<$ty, B> for Big {
            type Error = DecoderError;

            #[inline(always)]
            fn decode_from(self, buffer: B) -> Result<($ty, B), Self::Error> {
                buffer.slice_with(core::mem::size_of::<$ty>(), |slice| {
                    Ok($ty::from_be_bytes(
                        slice
                            .as_less_safe_slice()
                            .try_into()
                            .expect("length already checked"),
                    ))
                })
            }
        }

        impl<B: SliceableBuffer> Decoder<$ty, B> for Little {
            type Error = DecoderError;

            #[inline(always)]
            fn decode_from(self, buffer: B) -> Result<($ty, B), Self::Error> {
                buffer.slice_with(core::mem::size_of::<$ty>(), |slice| {
                    Ok($ty::from_le_bytes(
                        slice
                            .as_less_safe_slice()
                            .try_into()
                            .expect("length already checked"),
                    ))
                })
            }
        }
    };
}

impl_integer!(u16);
impl_integer!(i16);
impl_integer!(u32);
impl_integer!(i32);
impl_integer!(u64);
impl_integer!(i64);
impl_integer!(u128);
impl_integer!(i128);
impl_integer!(usize);
impl_integer!(isize);

macro_rules! impl_tuple {
    ($($T:ident),*) => {
        impl_tuple!([$($T,)*], []);
    };
    ([], [$($prev:ident),*]) => {
        // done
    };
    ([$current:ident, $($rest:ident,)*], [$($prev:ident),*]) => {
        impl<
            _B: SliceableBuffer,
            Error,
            $($prev: TypeDecoder<_B, Error = Error>,)*
            $current: TypeDecoder<_B, Error = Error>
        > TypeDecoder<_B> for ($($prev,)* $current,) {
            type Error = Error;

            #[inline(always)]
            fn decode_from(buffer: _B) -> Result<(Self, _B), Self::Error> {
                #![allow(non_snake_case)]
                $(
                    let ($prev, buffer) = buffer.decode()?;
                )*
                let ($current, buffer) = buffer.decode()?;
                let value = ($($prev,)* $current,);
                Ok((value, buffer))
            }
        }

        impl_tuple!([$($rest,)*], [$($prev,)* $current]);
    };
}

impl_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);

impl<B: SliceableBuffer> TypeDecoder<B> for () {
    type Error = DecoderError;

    #[inline(always)]
    fn decode_from(buffer: B) -> Result<(Self, B), Self::Error> {
        Ok(((), buffer))
    }
}

impl<B: FiniteBuffer, T: TypeDecoder<B>> TypeDecoder<B> for Option<T>
where
    T::Error: From<DecoderError>,
{
    type Error = T::Error;

    #[inline(always)]
    fn decode_from(buffer: B) -> Result<(Self, B), Self::Error> {
        if buffer.is_empty() {
            Ok((None, buffer))
        } else {
            let (value, buffer) = T::decode_from(buffer)?;
            Ok((Some(value), buffer))
        }
    }
}
