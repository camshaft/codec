use crate::{
    buffer::{Result, SplittableBuffer},
    decode::{DecoderBuffer, TypeDecoder},
    encode::{EncoderBuffer, TypeEncoder},
};
use std::net::{Ipv4Addr, Ipv6Addr};

macro_rules! addr {
    ($ty:ident, $prim:ident) => {
        impl<B: SplittableBuffer> TypeDecoder<B> for $ty {
            #[inline(always)]
            fn decode_type(buffer: B) -> Result<Self, B> {
                let (ip, buffer) = buffer.decode()?;
                let ip = $prim::into(ip);
                Ok((ip, buffer))
            }
        }

        impl<B: EncoderBuffer> TypeEncoder<B> for $ty {
            #[inline(always)]
            fn encode_type(self, buffer: B) -> Result<(), B> {
                let value: $prim = self.into();
                value.encode_type(buffer)
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
    };
}

addr!(Ipv4Addr, u32);
addr!(Ipv6Addr, u128);

/// asserts that the endian is correct
#[test]
fn ipv4_round_trip_test() {
    let localhost = &Ipv4Addr::LOCALHOST.octets()[..];
    let (actual, _) = localhost.decode::<Ipv4Addr>().unwrap();
    assert_eq!(actual, Ipv4Addr::LOCALHOST);

    let mut out = [0u8; 4];
    (&mut out[..]).encode(Ipv4Addr::LOCALHOST).unwrap();
    assert_eq!(out, localhost);
}
