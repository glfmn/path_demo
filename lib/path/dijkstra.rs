#[allow(clippy::unused_imports)]
use super::{Model, Optimizer, PathResult};
use radix_heap::{Radix, RadixHeapMap};

use std::fmt::Debug;

pub struct Dijkstra<M>
where
    M: Model,
    M::Cost: Raidx + Copy, {}

impl<M> Default for Dijkstra<M>
where
    M: Model,
    M::Cost: Radix + Copy,
{
    fn default() -> Self {
        Dijkstra {}
    }
}

impl<M> Optimizer for Dijkstra<M> {
    fn optimize(
        &mut self,
        model: &mut M,
        start: &M::State,
        goal: &M::State,
        sampler: &mut S,
    ) -> PathResult<M> {
        unimplemented!();
    }

    fn next_trajectory(
        &mut self,
        model: &mut M,
        start: &M::State,
        goal: &M::State,
        sampler: &mut S,
    ) -> PathResult<M> {
        unimplemented!();
    }
}
