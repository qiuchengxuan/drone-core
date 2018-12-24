use core::ops::{Generator, GeneratorState};
use fib::{Fiber, FiberRoot, FiberState};
use thr::prelude::*;

/// Generator fiber.
pub struct FiberGen<G>(G)
where
  G: Generator;

impl<G> Fiber for FiberGen<G>
where
  G: Generator,
{
  type Input = ();
  type Yield = G::Yield;
  type Return = G::Return;

  #[inline]
  fn resume(&mut self, _input: ()) -> FiberState<G::Yield, G::Return> {
    // FIXME Use `Pin` when implemented
    unsafe { self.0.resume().into() }
  }
}

impl<G> FiberRoot for FiberGen<G>
where
  G: Generator<Yield = (), Return = ()>,
  G: Send + 'static,
{
  #[inline]
  fn advance(&mut self) -> bool {
    match self.resume(()) {
      FiberState::Yielded(()) => true,
      FiberState::Complete(()) => false,
    }
  }
}

impl<Y, R> From<GeneratorState<Y, R>> for FiberState<Y, R> {
  #[inline]
  fn from(state: GeneratorState<Y, R>) -> Self {
    match state {
      GeneratorState::Yielded(val) => FiberState::Yielded(val),
      GeneratorState::Complete(val) => FiberState::Complete(val),
    }
  }
}

/// Creates a new generator fiber.
#[inline(always)]
pub fn new<G>(gen: G) -> FiberGen<G>
where
  G: Generator,
{
  FiberGen(gen)
}

/// Generator fiber extension to the thread token.
pub trait ThrFiberGen<T: ThrAttach>: ThrToken<T> {
  /// Adds a new generator fiber.
  fn add<G>(self, gen: G)
  where
    G: Generator<Yield = (), Return = ()>,
    G: Send + 'static,
  {
    self.add_fib(new(gen))
  }
}

impl<T: ThrAttach, U: ThrToken<T>> ThrFiberGen<T> for U {}