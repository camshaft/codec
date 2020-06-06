use crate::{
    buffer::{FiniteBuffer, FiniteMutBuffer, Result, SliceableBuffer},
    decode::{Decoder, TypeDecoder},
    endian::{Big, Little, NETWORK},
};
use core::convert::TryInto;

// encode::{Encoder, EncoderBuffer, EncoderCursor, TypeEncoder},
//

macro_rules! impl_byte {
    ($ty:ident) => {
        impl<B: SliceableBuffer> TypeDecoder<B> for $ty {
            #[inline(always)]
            fn decode_type(buffer: B) -> Result<Self, B> {
                buffer.slice_with(core::mem::size_of::<$ty>(), |slice| {
                    let v = slice.as_less_safe_slice()[0] as $ty;
                    Ok((v, slice))
                })
            }
        }

        // impl<B: EncoderBuffer> TypeEncoder<B> for $ty {
        //     fn encode_type(self, buffer: B) -> Result<EncoderCursor<$ty, B::Slice>, B> {
        //         buffer.encode_slice_with::<$ty, _>(|mut slice| {
        //             slice.as_less_safe_mut_slice()[0] = self as u8;
        //             Ok(((), slice))
        //         })
        //     }
        // }
    };
}

impl_byte!(u8);
impl_byte!(i8);

macro_rules! impl_integer {
    ($ty:ident) => {
        impl<B: SliceableBuffer> TypeDecoder<B> for $ty {
            #[inline(always)]
            fn decode_type(buffer: B) -> Result<Self, B> {
                NETWORK.decode_from(buffer)
            }
        }

        // impl<B: EncoderBuffer> TypeEncoder<B> for $ty
        // where
        //     B::Slice: FiniteMutBuffer,
        // {
        //     fn encode_type(self, buffer: B) -> Result<EncoderCursor<$ty, B::Slice>, B> {
        //         NETWORK.encode_into(self, buffer)
        //     }
        // }

        impl<B: SliceableBuffer> Decoder<$ty, B> for Big {
            #[inline(always)]
            fn decode_from(self, buffer: B) -> Result<$ty, B> {
                buffer.slice_with(core::mem::size_of::<$ty>(), |slice| {
                    let value = $ty::from_be_bytes(
                        slice
                            .as_less_safe_slice()
                            .try_into()
                            .expect("length already checked"),
                    );
                    Ok((value, slice))
                })
            }
        }

        // impl<B: EncoderBuffer> Encoder<$ty, B> for Big
        // where
        //     B::Slice: FiniteMutBuffer,
        // {
        //     fn encode_into(self, value: $ty, buffer: B) -> Result<EncoderCursor<$ty, B::Slice>, B> {
        //         buffer.encode_slice_with::<$ty, _>(|mut slice| {
        //             slice
        //                 .as_less_safe_mut_slice()
        //                 .copy_from_slice(&value.to_be_bytes());
        //             Ok(((), slice))
        //         })
        //     }
        // }

        impl<B: SliceableBuffer> Decoder<$ty, B> for Little {
            #[inline(always)]
            fn decode_from(self, buffer: B) -> Result<$ty, B> {
                buffer.slice_with(core::mem::size_of::<$ty>(), |slice| {
                    let value = $ty::from_le_bytes(
                        slice
                            .as_less_safe_slice()
                            .try_into()
                            .expect("length already checked"),
                    );
                    Ok((value, slice))
                })
            }
        }

        // impl<B: EncoderBuffer> Encoder<$ty, B> for Little
        // where
        //     B::Slice: FiniteMutBuffer,
        // {
        //     fn encode_into(self, value: $ty, buffer: B) -> Result<EncoderCursor<$ty, B::Slice>, B> {
        //         buffer.encode_slice_with::<$ty, _>(|mut slice| {
        //             slice
        //                 .as_less_safe_mut_slice()
        //                 .copy_from_slice(&value.to_le_bytes());
        //             Ok(((), slice))
        //         })
        //     }
        // }
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
impl_integer!(f32);
impl_integer!(f64);

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
            $($prev: TypeDecoder<_B>,)*
            $current: TypeDecoder<_B>
        > TypeDecoder<_B> for ($($prev,)* $current,) {
            #[inline(always)]
            fn decode_type(buffer: _B) -> Result<Self, _B> {
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
    #[inline(always)]
    fn decode_type(buffer: B) -> Result<Self, B> {
        Ok(((), buffer))
    }
}

impl<B: FiniteBuffer, T: TypeDecoder<B>> TypeDecoder<B> for Option<T> {
    #[inline(always)]
    fn decode_type(buffer: B) -> Result<Self, B> {
        if buffer.is_empty() {
            Ok((None, buffer))
        } else {
            let (value, buffer) = T::decode_type(buffer)?;
            Ok((Some(value), buffer))
        }
    }
}
