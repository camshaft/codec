use core::marker::PhantomData;

pub struct EncoderCursor<T, B> {
    value: PhantomData<T>,
    buffer: B,
}

impl<T, B> EncoderCursor<T, B> {
    #[inline(always)]
    pub fn new(buffer: B) -> Self {
        Self {
            value: PhantomData,
            buffer,
        }
    }
}
