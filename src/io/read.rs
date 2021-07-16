use alloc::boxed::Box;
use core::{future::Future, pin::Pin};

/// The `Read` trait allows for reading bytes from a source asynchronously.
pub trait Read<'sess, W, B: AsMut<[W]> + 'sess> {
    /// The error type returned by [`Read::read`].
    type Error;

    /// Pull some words from this source into the specified buffer
    /// asynchronously, eventually returning how many words were read.
    fn read(
        &'sess mut self,
        buffer: B,
    ) -> Pin<Box<dyn Future<Output = Result<usize, Self::Error>> + Send + 'sess>>;
}
