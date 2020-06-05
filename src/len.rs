use core::marker::PhantomData;

pub struct LenPrefix<L, T> {
    len: PhantomData<L>,
    t: PhantomData<T>,
}

pub struct LenSuffix<L, T> {
    len: PhantomData<L>,
    t: PhantomData<T>,
}

// impl<T: FromBytes + Unaligned, B: DecoderBuffer> Decoder<B> for LenSuffix<T, B> {
//     type Error = DecoderError;

//     fn decode_from(buffer: B) -> Result<(Self, B), Self::Error> {
//         let (owner, buffer) = buffer.split_at(size_of::<T>())?;
//         let value = Zerocopy {
//             buffer: owner,
//             value: PhantomData,
//         };
//         Ok((value, buffer))
//     }
// }
