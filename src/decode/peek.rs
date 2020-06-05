use crate::decode::{DecoderError, FiniteBuffer, FiniteMutBuffer, SliceableBuffer};

macro_rules! impl_peek {
    ($name:ident, [$($derive:ident),*], $a:lifetime, $ty:ty) => {
        #[derive($($derive,)* Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
        pub struct $name<$a>($ty);

        impl<$a> $name<$a> {
            pub fn new(buffer: $ty) -> Self {
                Self(buffer)
            }
        }

        impl<$a> SliceableBuffer for $name<$a> {
            type Slice = $name<$a>;

            #[inline(always)]
            fn slice(self, offset: usize) -> Result<(Self::Slice, Self), DecoderError> {
                let (a, b) = self.0.slice(offset)?;
                Ok(($name(a), $name(b)))
            }
        }

        impl<$a> FiniteBuffer for $name<$a> {
            #[inline(always)]
            fn as_less_safe_slice(&self) -> &[u8] {
                &self.0
            }
        }
    };
}

impl_peek!(PeekBuffer, [Clone, Copy], 'a, &'a [u8]);
impl_peek!(PeekMutBuffer, [], 'a, &'a mut [u8]);

impl<'a> FiniteMutBuffer for PeekMutBuffer<'a> {
    #[inline(always)]
    fn as_less_safe_slice_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}
