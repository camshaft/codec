use crate::buffer::{
    BufferError, FiniteBuffer, FiniteMutBuffer, PeekBuffer, PeekMutBuffer, Result, SliceableBuffer,
    SliceableMutBuffer,
};
use core::marker::PhantomData;

pub trait Encoder<T, B: EncoderBuffer>: Sized {
    fn encode_into(self, value: T, buffer: B) -> Result<EncoderCursor<T, B::Slice>, B>;
}

pub trait TypeEncoder<B: EncoderBuffer>: Sized {
    fn encode_type(self, buffer: B) -> Result<EncoderCursor<Self, B::Slice>, B>;
}

pub trait EncoderBuffer: Sized {
    type Slice: EncoderBuffer;

    fn encode<T: TypeEncoder<Self>>(self, value: T) -> Result<EncoderCursor<T, Self::Slice>, Self> {
        value.encode_type(self)
    }

    fn encode_with<T, E: Encoder<T, Self>>(
        self,
        value: T,
        encoder: E,
    ) -> Result<EncoderCursor<T, Self::Slice>, Self> {
        encoder.encode_into(value, self)
    }

    fn ensure_capacity(self, len: usize) -> Result<(), Self>;

    fn encode_bytes(self, bytes: &[u8]) -> Result<EncoderCursor<&[u8], Self::Slice>, Self>;

    fn encode_slice_with<T, F: FnOnce(Self) -> Result<(), Self>>(
        self,
        f: F,
    ) -> Result<EncoderCursor<T, Self::Slice>, Self>;
}

// pub trait EncoderSlice: Sized {
//     fn copy_from_slice<T>(self, bytes: &[u8]) -> Result<EncoderCursor<T, Self>, Self>;
// }

// impl<B: FiniteMutBuffer> EncoderBuffer for B
// where
//     B::Slice: FiniteMutBuffer,
// {
//     type Slice = B::Slice;

//     #[inline(always)]
//     fn encode_slice_with<
//         T,
//         F: FnOnce(BufferWriter<&mut Self>) -> Result<(), BufferWriter<&mut Self>>,
//     >(
//         mut self,
//         f: F,
//     ) -> Result<EncoderCursor<T, Self::Slice>, Self> {
//         let initial_len = self.len();

//         match f(BufferWriter(&mut self)) {
//             Ok(((), writer)) => {
//                 let consumed_len = initial_len
//                     .checked_sub(writer.len())
//                     .expect("invalid writer len");
//                 let (slice, buffer) = self.slice(consumed_len)?;
//                 let cursor = EncoderCursor {
//                     value: PhantomData,
//                     buffer: slice,
//                 };
//                 Ok((cursor, buffer))
//             }
//             Err(err) => Err(BufferError {
//                 reason: err.reason,
//                 buffer: self,
//             }),
//         }
//     }
// }

pub struct BufferWriter<Buffer>(Buffer);

// pub trait EncoderBuffer: Sized {
//     type Slice: FiniteMutBuffer;

//     fn encode<T: TypeEncoder<Self>>(self, value: T) -> Result<EncoderCursor<T, Self::Slice>, Self> {
//         value.encode_type(self)
//     }

//     fn encode_with<T, E: Encoder<T, Self>>(
//         self,
//         value: T,
//         encoder: E,
//     ) -> Result<EncoderCursor<T, Self::Slice>, Self> {
//         encoder.encode_into(value, self)
//     }

//     #[inline(always)]
//     fn encode_sized_with<T: Sized, F: FnOnce(PeekMutBuffer) -> Result<(), PeekMutBuffer>>(
//         self,
//         f: F,
//     ) -> Result<EncoderCursor<T, Self::Slice>, Self> {
//         let (mut a, b) = self.slice(core::mem::size_of::<T>())?;
//         let res = f(PeekMutBuffer::new(a.as_less_safe_mut_slice()));
//         let (_, b) = map_buffer_error!(res, b);
//         let a = EncoderCursor {
//             value: PhantomData,
//             buffer: a,
//         };
//         Ok((a, b))
//     }

//     #[inline(always)]
//     fn encode_len_with<T, F: FnOnce(PeekMutBuffer) -> Result<(), PeekMutBuffer>>(
//         self,
//         len: usize,
//         f: F,
//     ) -> Result<EncoderCursor<T, Self::Slice>, Self> {
//         let (mut a, b) = self.slice(core::mem::size_of::<T>())?;
//         let res = f(PeekMutBuffer::new(a.as_less_safe_mut_slice()));
//         let (_, b) = map_buffer_error!(res, b);
//         let a = EncoderCursor {
//             value: PhantomData,
//             buffer: a,
//         };
//         Ok((a, b))
//     }
// }

pub struct EncoderCursor<T, B> {
    value: PhantomData<T>,
    buffer: B,
}

#[derive(Clone, Copy, Debug)]
pub struct FiniteEncoderBuffer<B> {
    buffer: B,
}

impl<B: FiniteMutBuffer> FiniteEncoderBuffer<B> {
    pub fn new(buffer: B) -> Self {
        Self { buffer }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct LenEstimator {
    start: usize,
    end: usize,
}

impl SliceableBuffer for LenEstimator {
    type Slice = LenEstimator;

    #[inline(always)]
    fn slice(self, len: usize) -> Result<Self::Slice, Self> {
        let a = LenEstimator {
            start: self.start,
            end: self.start + len,
        };
        let b = LenEstimator {
            start: a.end,
            end: self.end,
        };
        Ok((a, b))
    }

    #[inline(always)]
    fn slice_with<T, F: FnOnce(PeekBuffer) -> Result<T, PeekBuffer>>(
        self,
        _len: usize,
        _f: F,
    ) -> Result<T, Self> {
        panic!("cannot slice_with LenEstimator");
    }
}

impl SliceableMutBuffer for LenEstimator {
    type FrozenSlice = LenEstimator;

    fn freeze(self) -> Self::FrozenSlice {
        self
    }
}

impl FiniteBuffer for LenEstimator {
    fn as_less_safe_slice(&self) -> &[u8] {
        panic!("cannot convert LenEstimator into slice");
    }
}

impl FiniteMutBuffer for LenEstimator {
    fn as_less_safe_mut_slice(&mut self) -> &mut [u8] {
        panic!("cannot convert LenEstimator into mut slice");
    }
}
