use super::{Model, Optimizer, PathFindingErr, PathResult, Sampler, State, Trajectory};
use fnv::FnvHashMap;
use radix_heap::{Radix, RadixHeapMap};

use std::cmp::{PartialEq, Reverse};
use std::collections::hash_map::Entry;
use std::fmt::{self, Debug, Formatter};
use std::hash::{self, Hash};

pub struct Dijkstra<M>
where
    M: Model,
    M::Cost: Radix + Copy,
{
    queue: RadixHeapMap<M::Cost, Node<M>>,
    grid: FnvHashMap<<<M as Model>::State as State>::Position, Id<M>>,
    parent_map: FnvHashMap<Id<M>, Node<M>>,
    id_counter: usize,
}

impl<M> Default for Dijkstra<M>
where
    M: Model,
    M::Cost: Radix + Copy,
{
    fn default() -> Self {
        Dijkstra {
            queue: Default::default(),
            grid: Default::default(),
            parent_map: Default::default(),
            id_counter: 0,
        }
    }
}

impl<M> Dijkstra<M>
where
    M: Model,
    M::Cost: Radix + Copy,
{
    #[inline(always)]
    fn step<S>(
        &mut self,
        current: &Node<M>,
        model: &mut M,
        goal: &M::State,
        sampler: &mut S,
    ) -> bool
    where
        S: Sampler<M>,
    {
        if model.converge(&current.state, goal) {
            return true;
        }

        for control in sampler.sample(model, &current.state) {
            if let Some(child_state) = model.integrate(&current.state, &control) {
                self.id_counter += 1;

                let cost = current.id.g.0 + model.cost(&current.state, &control, &child_state);

                let child = Node::<M> {
                    id: Id::new(self.id_counter, cost),
                    state: child_state,
                    control: control.clone(),
                };

                let position = self.grid.entry(child.state.grid_position());

                match position {
                    Entry::Occupied(mut best) => {
                        let best = best.get_mut();
                        if best.g.0 <= child.id.g.0 {
                            continue;
                        } else {
                            *best = child.id.clone();
                        }
                    }
                    Entry::Vacant(empty) => {
                        empty.insert(child.id.clone());
                    }
                }

                self.parent_map.insert(child.id.clone(), current.clone());
                self.queue.push(child.id.g.0, child);
            }
        }

        false
    }

    fn unwind_trajectory(&self, mut current: Node<M>) -> Trajectory<M> {
        let mut result = Vec::new();
        result.push((current.state.clone(), current.control.clone()));

        while let Some(p) = self.parent_map.get(&current.id) {
            current = (*p).clone();
            result.push((current.state.clone(), current.control.clone()));
        }

        Trajectory { cost: current.id.g.0, trajectory: result }
    }
}

impl<M, S> Optimizer<M, S> for Dijkstra<M>
where
    M: Model,
    M::Cost: Copy + Radix,
    S: Sampler<M>,
{
    fn optimize(
        &mut self,
        model: &mut M,
        start: &M::State,
        goal: &M::State,
        sampler: &mut S,
    ) -> PathResult<M> {
        use PathFindingErr::*;
        use PathResult::*;

        if model.converge(start, goal) {
            return Final(Trajectory {
                cost: Default::default(),
                trajectory: vec![(start.clone(), Default::default())],
            });
        }

        if self.queue.top().is_none() {
            let start_id = Id::new(0, Default::default());
            self.queue.push(
                Default::default(),
                Node { id: start_id, state: start.clone(), control: Default::default() },
            );
        }

        while let Some((_, current)) = self.queue.pop() {
            if self.step(&current, model, &goal, sampler) {
                return Final(self.unwind_trajectory(current));
            }
        }

        Err(Unreachable)
    }

    fn next_trajectory(
        &mut self,
        model: &mut M,
        start: &M::State,
        goal: &M::State,
        sampler: &mut S,
    ) -> PathResult<M> {
        use PathFindingErr::*;
        use PathResult::*;

        if self.parent_map.is_empty() && self.queue.is_empty() {
            let start_id = Id::new(0, Default::default());
            self.queue.push(
                Default::default(),
                Node { id: start_id, state: start.clone(), control: Default::default() },
            );
        }

        if let Some((_, current)) = self.queue.pop() {
            if self.step(&current, model, &goal, sampler) {
                Final(self.unwind_trajectory(current))
            } else {
                Intermediate(self.unwind_trajectory(current))
            }
        } else {
            Err(Unreachable)
        }
    }
}

struct Id<M>
where
    M: Model,
{
    id: usize,
    g: Reverse<M::Cost>,
}

impl<M> Id<M>
where
    M: Model,
{
    fn new(id: usize, g: M::Cost) -> Self {
        Id { id, g: Reverse(g) }
    }
}

impl<M> PartialEq for Id<M>
where
    M: Model,
{
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<M> Eq for Id<M> where M: Model {}

impl<M> Hash for Id<M>
where
    M: Model,
{
    fn hash<H: hash::Hasher>(&self, hasher: &mut H) {
        self.id.hash(hasher);
    }
}

impl<M> Clone for Id<M>
where
    M: Model,
{
    fn clone(&self) -> Self {
        Id::new(self.id, self.g.0.clone())
    }
}

impl<M> Debug for Id<M>
where
    M: Model,
    M::Cost: Debug,
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("Id").field("id", &self.id).field("g", &self.g).finish()
    }
}

struct Node<M>
where
    M: Model,
{
    id: Id<M>,
    state: M::State,
    control: M::Control,
}

impl<M> Clone for Node<M>
where
    M: Model,
{
    fn clone(&self) -> Self {
        Node { id: self.id.clone(), state: self.state.clone(), control: self.control.clone() }
    }
}

impl<M> Debug for Node<M>
where
    M: Model,
    M::State: Debug,
    M::Control: Debug,
    M::Cost: Debug,
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("Node")
            .field("id", &self.id)
            .field("state", &self.state)
            .field("control", &self.control)
            .finish()
    }
}
