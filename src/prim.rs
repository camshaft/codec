use crate::{
    buffer::{FiniteBuffer, Result, SplittableBuffer},
    decode::{Decoder, TypeDecoder},
    encode::{Encoder, EncoderBuffer, TypeEncoder},
    endian::{Big, Little, NETWORK},
};
use core::convert::TryInto;

macro_rules! impl_int_tests {
    ($ty:ident, $tests:ident) => {
        #[cfg(test)]
        mod $tests {
            use super::*;
            use core::mem::size_of;

            #[test]
            fn round_trip_test() {
                let mut buffer = [0; size_of::<$ty>()];
                let slice = &mut buffer[..];

                macro_rules! round_trip {
                    ($value: expr) => {
                        let value: $ty = $value;
                        assert_eq!(slice.encoding_len(&value), Ok(size_of::<$ty>()));
                        slice.encode::<$ty>(value).unwrap();
                        let (v, _) = slice.decode::<$ty>().unwrap();
                        assert_eq!(v, value);
                    };
                }

                round_trip!(1);
                round_trip!($ty::MIN);
                round_trip!($ty::MIN + 1);
                round_trip!($ty::MAX / 2);
                round_trip!($ty::MAX - 1);
                round_trip!($ty::MAX);
            }
        }
    };
}

macro_rules! impl_byte {
    ($ty:ident, $tests:ident) => {
        impl<B: SplittableBuffer> TypeDecoder<B> for $ty {
            #[inline(always)]
            fn decode_type(buffer: B) -> Result<Self, B> {
                buffer.checked_split_with(core::mem::size_of::<$ty>(), |slice| {
                    let v = slice.as_less_safe_slice()[0] as $ty;
                    Ok((v, slice))
                })
            }
        }

        impl<B> TypeEncoder<B> for $ty
        where
            B: EncoderBuffer,
        {
            #[inline(always)]
            fn encode_type(self, buffer: B) -> Result<(), B> {
                let (_, buffer) = buffer.encode_bytes([self as u8])?;
                Ok(((), buffer))
            }
        }

        impl<B: EncoderBuffer> TypeEncoder<B> for &$ty {
            #[inline(always)]
            fn encode_type(self, buffer: B) -> Result<(), B> {
                (*self).encode_type(buffer)
            }
        }

        impl<B: EncoderBuffer> TypeEncoder<B> for &mut $ty {
            #[inline(always)]
            fn encode_type(self, buffer: B) -> Result<(), B> {
                (*self).encode_type(buffer)
            }
        }

        impl_int_tests!($ty, $tests);
    };
}

impl_byte!(u8, u8_tests);
impl_byte!(i8, i8_tests);

macro_rules! impl_integer {
    ($ty:ident $(, $tests:ident)?) => {
        impl<B: SplittableBuffer> TypeDecoder<B> for $ty {
            #[inline(always)]
            fn decode_type(buffer: B) -> Result<Self, B> {
                NETWORK.decode_from(buffer)
            }
        }

        impl<B: EncoderBuffer> TypeEncoder<B> for $ty {
            #[inline(always)]
            fn encode_type(self, buffer: B) -> Result<(), B> {
                NETWORK.encode_into(self, buffer)
            }
        }

        impl<B: EncoderBuffer> TypeEncoder<B> for &$ty {
            #[inline(always)]
            fn encode_type(self, buffer: B) -> Result<(), B> {
                (*self).encode_type(buffer)
            }
        }

        impl<B: EncoderBuffer> TypeEncoder<B> for &mut $ty {
            #[inline(always)]
            fn encode_type(self, buffer: B) -> Result<(), B> {
                (*self).encode_type(buffer)
            }
        }

        impl<B: SplittableBuffer> Decoder<$ty, B> for Big {
            #[inline(always)]
            fn decode_from(self, buffer: B) -> Result<$ty, B> {
                buffer.checked_split_with(core::mem::size_of::<$ty>(), |slice| {
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

        impl<B: EncoderBuffer> Encoder<$ty, B> for Big {
            #[inline(always)]
            fn encode_into(self, value: $ty, buffer: B) -> Result<(), B> {
                let (_, buffer) = buffer.encode_bytes(value.to_be_bytes())?;
                Ok(((), buffer))
            }
        }

        impl<B: EncoderBuffer> Encoder<&$ty, B> for Big {
            #[inline(always)]
            fn encode_into(self, value: &$ty, buffer: B) -> Result<(), B> {
                self.encode_into(*value, buffer)
            }
        }

        impl<B: EncoderBuffer> Encoder<&mut $ty, B> for Big {
            #[inline(always)]
            fn encode_into(self, value: &mut $ty, buffer: B) -> Result<(), B> {
                self.encode_into(*value, buffer)
            }
        }

        impl<B: SplittableBuffer> Decoder<$ty, B> for Little {
            #[inline(always)]
            fn decode_from(self, buffer: B) -> Result<$ty, B> {
                buffer.checked_split_with(core::mem::size_of::<$ty>(), |slice| {
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

        impl<B: EncoderBuffer> Encoder<$ty, B> for Little {
            #[inline(always)]
            fn encode_into(self, value: $ty, buffer: B) -> Result<(), B> {
                let (_, buffer) = buffer.encode_bytes(value.to_be_bytes())?;
                Ok(((), buffer))
            }
        }

        impl<B: EncoderBuffer> Encoder<&$ty, B> for Little {
            #[inline(always)]
            fn encode_into(self, value: &$ty, buffer: B) -> Result<(), B> {
                self.encode_into(*value, buffer)
            }
        }

        impl<B: EncoderBuffer> Encoder<&mut $ty, B> for Little {
            #[inline(always)]
            fn encode_into(self, value: &mut $ty, buffer: B) -> Result<(), B> {
                self.encode_into(*value, buffer)
            }
        }

        $(
            impl_int_tests!($ty, $tests);
        )*
    };
}

impl_integer!(u16, u16_tests);
impl_integer!(i16, i16_tests);
impl_integer!(u32, u32_tests);
impl_integer!(i32, i32_tests);
impl_integer!(u64, u64_tests);
impl_integer!(i64, i64_tests);
impl_integer!(u128, u128_tests);
impl_integer!(i128, i128_tests);
impl_integer!(usize, usize_tests);
impl_integer!(isize, isize_tests);
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
            _B: SplittableBuffer,
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

        impl<
            _B: EncoderBuffer,
            $($prev: TypeEncoder<_B>,)*
            $current: TypeEncoder<_B>
        > TypeEncoder<_B> for ($($prev,)* $current,) {
            #[inline(always)]
            fn encode_type(self, buffer: _B) -> Result<(), _B> {
                #![allow(non_snake_case)]
                let ($($prev ,)* $current,) = self;
                $(
                    let (_, buffer) = buffer.encode($prev)?;
                )*
                let (_, buffer) = buffer.encode($current)?;
                Ok(((), buffer))
            }
        }

        impl<'a,
            _B: EncoderBuffer,
            $($prev,)*
            $current
        > TypeEncoder<_B> for &'a ($($prev,)* $current,)
        where
            $(&'a $prev: TypeEncoder<_B>,)*
            &'a $current: TypeEncoder<_B>
        {
            #[inline(always)]
            fn encode_type(self, buffer: _B) -> Result<(), _B> {
                #![allow(non_snake_case)]
                let ($($prev ,)* $current,) = self;
                $(
                    let (_, buffer) = buffer.encode($prev)?;
                )*
                let (_, buffer) = buffer.encode($current)?;
                Ok(((), buffer))
            }
        }

        impl<'a,
            _B: EncoderBuffer,
            $($prev,)*
            $current
        > TypeEncoder<_B> for &'a mut ($($prev,)* $current,)
        where
            $(&'a mut $prev: TypeEncoder<_B>,)*
            &'a mut $current: TypeEncoder<_B>
        {
            #[inline(always)]
            fn encode_type(self, buffer: _B) -> Result<(), _B> {
                #![allow(non_snake_case)]
                let ($($prev ,)* $current,) = self;
                $(
                    let (_, buffer) = buffer.encode($prev)?;
                )*
                let (_, buffer) = buffer.encode($current)?;
                Ok(((), buffer))
            }
        }

        impl_tuple!([$($rest,)*], [$($prev,)* $current]);
    };
}

impl_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);

impl<B: SplittableBuffer> TypeDecoder<B> for () {
    #[inline(always)]
    fn decode_type(buffer: B) -> Result<Self, B> {
        Ok(((), buffer))
    }
}

impl<B: EncoderBuffer> TypeEncoder<B> for () {
    #[inline(always)]
    fn encode_type(self, buffer: B) -> Result<(), B> {
        Ok(((), buffer))
    }
}

impl<B: EncoderBuffer> TypeEncoder<B> for &() {
    #[inline(always)]
    fn encode_type(self, buffer: B) -> Result<(), B> {
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

impl<B: EncoderBuffer, T: TypeEncoder<B>> TypeEncoder<B> for Option<T> {
    #[inline(always)]
    fn encode_type(self, buffer: B) -> Result<(), B> {
        if let Some(value) = self {
            let (_, buffer) = buffer.encode(value)?;
            Ok(((), buffer))
        } else {
            Ok(((), buffer))
        }
    }
}

impl<'a, B: EncoderBuffer, T> TypeEncoder<B> for &'a Option<T>
where
    &'a T: TypeEncoder<B>,
{
    #[inline(always)]
    fn encode_type(self, buffer: B) -> Result<(), B> {
        if let Some(value) = self {
            let (_, buffer) = buffer.encode(value)?;
            Ok(((), buffer))
        } else {
            Ok(((), buffer))
        }
    }
}
