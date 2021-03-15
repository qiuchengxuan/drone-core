use core::{future::Future, pin::Pin};

/// The `Write` trait allows for writing bytes to a source asynchronously.
pub trait Write<'sess, W, B: AsRef<[W]> + 'sess> {
    /// The error type returned by [`Write::write`].
    type Error;

    /// Write some words into this writer asynchronously, eventually returning how
    /// many words were written.
    fn write(
        &'sess mut self,
        words: B,
    ) -> Pin<Box<dyn Future<Output = Result<usize, Self::Error>> + Send + 'sess>>;
}
