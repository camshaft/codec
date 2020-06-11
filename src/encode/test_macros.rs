macro_rules! encoder_buffer_tests {
    ($($arg:tt)*) => {
        __encoder_buffer_test!(encode_u8_test, 1, [$($arg)*], |buffer, _capacity| {
            let (written, buffer) = buffer.encode(123u8).expect("capacity");
            assert_eq!(written, 1, "incorrect written report");
            buffer
        }, |buffer, _capacity| {
            assert_eq!(buffer, &[123][..]);
        });

        __encoder_buffer_test!(encode_u16_test, 2, [$($arg)*], |buffer, _capacity| {
            let (written, buffer) = buffer.encode(123u16).expect("capacity");
            assert_eq!(written, 2, "incorrect written report");
            buffer
        }, |buffer, _capacity| {
            assert_eq!(buffer, &[0, 123][..]);
        });

        __encoder_buffer_test!(encode_u8_eof_test, 0, [$($arg)*], |buffer, capacity| {
            if capacity > 0 {
                let (written, buffer) = buffer.encode(123u8).expect("capacity");
                assert_eq!(written, 1, "incorrect written report");
                buffer
            } else {
                let err = buffer.encode(123u8).unwrap_err();
                err.buffer
            }
        }, |buffer, capacity| {
            if capacity > 0 {
                assert_eq!(buffer, &[123][..]);
            } else {
                assert_eq!(buffer, &[][..]);
            }
        });

        __encoder_buffer_test!(encode_u16_eof_test, 1, [$($arg)*], |buffer, capacity| {
            if capacity > 1 {
                let (written, buffer) = buffer.encode(123u16).expect("capacity");
                assert_eq!(written, 2, "incorrect written report");
                buffer
            } else {
                let err = buffer.encode(123u16).unwrap_err();
                assert!(err.buffer.encoder_capacity() == capacity, "incorrect capacity report");
                err.buffer
            }
        }, |buffer, capacity| {
            if capacity > 1 {
                assert_eq!(buffer, &[0, 123][..]);
            } else {
                assert_eq!(buffer, &[0][..]);
            }
        });
    };
}

macro_rules! __encoder_buffer_test {
    (
        $name:ident,
        $size:expr,
        [
            $ty:ty,
            | $len:ident, $buffer:ident | { $($init:stmt;)* }
            $(, | $as_ref_buf:ident | $as_ref:expr)?
        ],
        $test:expr,
        $buffer_test:expr
    ) => {
      #[test]
        fn $name() {
            #![allow(non_upper_case_globals)]

            use crate::encode::EncoderBuffer;

            const $len: usize = $size;
            let $buffer;
            $($init)*

            let encoder_capacity = $buffer.encoder_capacity();
            assert!(encoder_capacity >= $len, "invalid initial buffer");

            let test: fn($ty, usize) -> $ty = $test;

            #[allow(unused_variables)]
            let test_result = test($buffer, encoder_capacity);

            #[allow(unused_variables)]
            let buffer_test: fn(&[u8], usize) = $buffer_test;

            $(
                #[allow(unused_mut)]
                let mut $as_ref_buf = test_result;
                buffer_test($as_ref, encoder_capacity);
            )?
        }
    };
}
