mod buffer;
mod decoder;
mod peek;
mod slice;

pub use buffer::*;
pub use decoder::*;
pub use peek::*;
pub use slice::*;

#[derive(Clone, Copy, Debug)]
pub enum DecoderError {
    UnexpectedEof { actual: usize, expected: usize },
    UnexpectedBytes { len: usize },
}

#[cfg(test)]
mod tests {
    use super::{FiniteBuffer, SliceableBuffer};
    use crate::bytes::*;

    fn peek_check<D: FiniteBuffer>(d: D) {
        {
            let t = d.peek();
            let (_, t) = t.decode::<u8>().unwrap();
            let (_, t) = t.decode::<u8>().unwrap();
            let _ = t;
        }
        decode_check(d)
    }

    fn decode_check<S: SliceableBuffer>(s: S) {
        let (_, s) = s.decode::<u8>().unwrap();
        let (_, s) = s.decode::<u8>().unwrap();
        let _ = s;
    }

    #[test]
    fn slice_test() {
        let mut i = [1u8, 2, 3];
        peek_check(&i[..]);
        peek_check(&mut i[..]);
        peek_check(Bytes::from(vec![1, 2, 3]));
    }
}
