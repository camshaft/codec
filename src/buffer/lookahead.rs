use crate::{
    buffer::{
        BorrowedBuffer, BorrowedMutBuffer, BufferError, FiniteBuffer, FiniteMutBuffer, Result,
        SplittableBuffer,
    },
    encode::EncoderBuffer,
};

macro_rules! impl_lookahead {
    ($name:ident, [$($derive:ident),*], $a:lifetime, $ty:ty) => {
        #[derive($($derive,)* Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
        pub struct $name<$a>($ty);

        impl<$a> $name<$a> {
            #[inline(always)]
            pub fn new(buffer: $ty) -> Self {
                Self(buffer)
            }
        }

        impl<$a> SplittableBuffer for $name<$a> {
            type Slice = $name<$a>;

            #[inline(always)]
            fn checked_split(self, offset: usize) -> Result<Self::Slice, Self> {
                let (a, b) = self.0.checked_split(offset).map_err(|err| err.map_buffer($name))?;
                Ok(($name(a), $name(b)))
            }
        }

        impl<$a> FiniteBuffer for $name<$a> {
            #[inline(always)]
            fn as_less_safe_slice(&self) -> &[u8] {
                &self.0
            }
        }

        impl<$a> BorrowedBuffer<$a> for $name<$a> {
            fn into_less_safe_slice(self) -> &$a [u8] {
                self.0
            }
        }
    };
}

impl_lookahead!(LookaheadBuffer, [Clone, Copy], 'a, &'a [u8]);
impl_lookahead!(LookaheadMutBuffer, [], 'a, &'a mut [u8]);

impl<'a> FiniteMutBuffer for LookaheadMutBuffer<'a> {
    #[inline(always)]
    fn as_less_safe_mut_slice(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

impl<'a> BorrowedMutBuffer<'a> for LookaheadMutBuffer<'a> {
    fn into_less_safe_mut_slice(self) -> &'a mut [u8] {
        self.0
    }
}

impl<'a> EncoderBuffer for LookaheadMutBuffer<'a> {
    #[inline(always)]
    fn encoder_capacity(&self) -> usize {
        self.len()
    }

    #[inline(always)]
    fn encode_bytes<T: AsRef<[u8]>>(self, bytes: T) -> Result<usize, Self> {
        let bytes = bytes.as_ref();
        let len = bytes.len();
        let (mut slice, buffer) = self.checked_split(len)?;
        slice.as_less_safe_mut_slice().copy_from_slice(bytes);
        Ok((len, buffer))
    }

    #[inline(always)]
    fn checkpoint<F>(self, f: F) -> Result<usize, Self>
    where
        F: FnOnce(Self) -> Result<(), Self>,
    {
        let res = self.0.checkpoint(|buffer| match f(Self(buffer)) {
            Ok(((), buffer)) => Ok(((), buffer.0)),
            Err(err) => Err(BufferError {
                buffer: err.buffer.0,
                reason: err.reason,
            }),
        });

        match res {
            Ok((len, buffer)) => Ok((len, Self(buffer))),
            Err(err) => Err(BufferError {
                buffer: Self(err.buffer),
                reason: err.reason,
            }),
        }
    }
}
