#[macro_use]
pub mod buffer;

pub mod decode;
pub mod encode;
pub mod endian;
pub mod len;
pub mod prim;
pub mod slice;

#[cfg(feature = "bytes")]
pub mod bytes;

#[cfg(feature = "zerocopy")]
pub mod zerocopy;
