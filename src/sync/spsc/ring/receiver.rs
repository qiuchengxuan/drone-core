use super::{Inner, COMPLETE, NUMBER_BITS, NUMBER_MASK};
use crate::sync::spsc::{SpscInner, SpscInnerErr};
use alloc::sync::Arc;
use core::{
    pin::Pin,
    ptr,
    sync::atomic::Ordering,
    task::{Context, Poll},
};
use futures::stream::Stream;

const IS_TX_HALF: bool = false;

/// The receiving-half of [`ring::channel`](super::channel).
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct Receiver<T, E> {
    inner: Arc<Inner<T, E>>,
}

impl<T, E> Receiver<T, E> {
    pub(super) fn new(inner: Arc<Inner<T, E>>) -> Self {
        Self { inner }
    }

    /// Gracefully close this receiver, preventing any subsequent attempts to
    /// send to it.
    ///
    /// Any `send` operation which happens after this method returns is
    /// guaranteed to fail. After calling this method, you can use
    /// [`Receiver::poll`](core::future::Future::poll) to determine whether a
    /// message had previously been sent.
    #[inline]
    pub fn close(&mut self) {
        self.inner.close_half(IS_TX_HALF)
    }

    /// Attempts to receive a value outside of the context of a task.
    ///
    /// Does not schedule a task wakeup or have any other side effects.
    ///
    /// A return value of `Ok(None)` must be considered immediately stale (out
    /// of date) unless [`close`](Receiver::close) has been called first.
    #[inline]
    pub fn try_next(&mut self) -> Result<Option<T>, E> {
        self.inner.try_next()
    }
}

impl<T, E> Stream for Receiver<T, E> {
    type Item = Result<T, E>;

    #[inline]
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.inner.poll_half_with_transaction(
            cx,
            IS_TX_HALF,
            Ordering::Acquire,
            Ordering::AcqRel,
            Inner::take_index_try,
            Inner::take_index_finalize,
        )
    }
}

impl<T, E> Drop for Receiver<T, E> {
    #[inline]
    fn drop(&mut self) {
        self.inner.close_half(IS_TX_HALF);
    }
}

impl<T, E> Inner<T, E> {
    pub(super) fn take_index(&self, state: &mut usize, length: usize) -> usize {
        let cursor = *state >> NUMBER_BITS & NUMBER_MASK;
        *state >>= NUMBER_BITS << 1;
        *state <<= NUMBER_BITS;
        *state |= cursor.wrapping_add(1).wrapping_rem(self.buffer.capacity());
        *state <<= NUMBER_BITS;
        *state |= length.wrapping_sub(1);
        cursor
    }

    pub(super) fn get_length(state: usize) -> usize {
        state & NUMBER_MASK
    }

    fn try_next(&self) -> Result<Option<T>, E> {
        let state = self.state_load(Ordering::Acquire);
        self.transaction(state, Ordering::AcqRel, Ordering::Acquire, |state| {
            match self.take_index_try(state) {
                Some(value) => value.map_err(Ok),
                None => Err(Err(())),
            }
        })
        .map(|index| unsafe { Some(self.take_value(index)) })
        .or_else(|value| value.map_or_else(|()| Ok(None), |()| self.take_err().transpose()))
    }

    fn take_index_try(&self, state: &mut usize) -> Option<Result<usize, ()>> {
        let length = Self::get_length(*state);
        if length != 0 {
            Some(Ok(self.take_index(state, length)))
        } else if *state & COMPLETE == 0 {
            None
        } else {
            Some(Err(()))
        }
    }

    fn take_index_finalize(&self, value: Result<usize, ()>) -> Option<Result<T, E>> {
        match value {
            Ok(index) => unsafe { Some(Ok(self.take_value(index))) },
            Err(()) => self.take_err(),
        }
    }

    unsafe fn take_value(&self, index: usize) -> T {
        unsafe { ptr::read(ptr::addr_of!(self.buffer[index])) }
    }
}
