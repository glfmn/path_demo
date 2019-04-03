use std::fmt::{Debug, Formatter};

use fnv::FnvHashMap;
use radix_heap::RadixHeapMap;
use std::cmp::{Ord, Ordering, PartialEq, PartialOrd, Reverse};
use std::collections::hash_map::Entry;
use std::hash::{Hash, Hasher};

use super::*;

pub struct AStar<M>
where
    M: HeuristicModel,
    M::Cost: radix_heap::Radix + Copy,
{
    queue: RadixHeapMap<Reverse<M::Cost>, Node<M>>,
    parent_map: FnvHashMap<Id<M>, Node<M>>,
    grid: FnvHashMap<<<M as Model>::State as State>::Position, Id<M>>,
    id_counter: usize,
}

impl<M> AStar<M>
where
    M: HeuristicModel,
    M::Cost: radix_heap::Radix + Copy,
{
    /// Create a new AStar optimizer
    pub fn new() -> Self {
        AStar {
            queue: RadixHeapMap::new(),
            parent_map: FnvHashMap::default(),
            grid: FnvHashMap::default(),
            id_counter: 0,
        }
    }

    pub fn clear(&mut self) {
        self.queue.clear();
        self.parent_map.clear();
        self.grid.clear();
    }

    pub fn inspect_queue(&self) -> impl Iterator<Item = (&M::State, &M::Control)> {
        self.queue.values().map(|node| (&node.state, &node.control))
    }

    pub fn inspect_discovered(
        &self,
    ) -> impl Iterator<Item = &<<M as Model>::State as State>::Position> {
        self.grid.keys()
    }

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

                let cost = current.id.g() + model.cost(&current.state, &control, &child_state);
                let heuristic = model.heuristic(&child_state, goal);

                let child = Node::<M> {
                    id: Id::new(self.id_counter, cost + heuristic, cost),
                    state: child_state,
                    control: control.clone(),
                };

                let position = self.grid.entry(child.state.grid_position());

                match position {
                    Entry::Occupied(mut best) => {
                        let best = best.get_mut();
                        if best.g <= child.id.g {
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
                self.queue.push(child.id.f, child);
            }
        }

        false
    }

    /// Follow the parents from the goal node up to the start node
    fn unwind_trajectory(&self, model: &M, mut current: Node<M>) -> Trajectory<M> {
        let mut result = Vec::new();
        result.push((current.state.clone(), current.control.clone()));
        let mut cost = M::Cost::default();

        // build up the trajectory by following the parent nodes
        while let Some(p) = self.parent_map.get(&current.id) {
            cost = cost + model.cost(&current.state, &current.control, &p.state);
            current = (*p).clone();
            result.push((current.state.clone(), current.control.clone()));
        }

        result.reverse();

        Trajectory { cost, trajectory: result }
    }
}

impl<M, S> Optimizer<M, S> for AStar<M>
where
    M: HeuristicModel,
    M::Cost: radix_heap::Radix + Copy,
    S: Sampler<M>,
{
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
            let heuristic = model.heuristic(start, goal);
            let start_id = Id::new(0, heuristic, Default::default());
            self.queue.push(
                Default::default(),
                Node { id: start_id, state: start.clone(), control: Default::default() },
            );
        }

        if let Some((_, current)) = self.queue.pop() {
            if self.step(&current, model, &goal, sampler) {
                Final(self.unwind_trajectory(model, current))
            } else {
                Intermediate(self.unwind_trajectory(model, current))
            }
        } else {
            Err(Unreachable)
        }
    }

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
            let start_id = Id::new(0, model.heuristic(start, goal), Default::default());
            self.queue.push(
                Default::default(),
                Node { id: start_id, state: start.clone(), control: Default::default() },
            );
        }

        while let Some((_, current)) = self.queue.pop() {
            if self.step(&current, model, &goal, sampler) {
                return Final(self.unwind_trajectory(model, current));
            }
        }

        Err(Unreachable)
    }
}

impl<M> Debug for AStar<M>
where
    M: HeuristicModel,
    M::State: Debug,
    M::Control: Debug,
    M::Cost: Debug + radix_heap::Radix + Copy,
{
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), std::fmt::Error> {
        fmt.debug_struct("AStar")
            .field("counter", &self.id_counter)
            .field("next", &self.queue.top())
            .field("queue", &self.queue)
            .field("grid", &self.grid)
            .field("parent_map", &self.parent_map)
            .finish()
    }
}

impl<M> Default for AStar<M>
where
    M: HeuristicModel,
    M::Cost: radix_heap::Radix + Copy,
{
    fn default() -> Self {
        Self::new()
    }
}

/// The Id which identifies a particular node and allows for comparisons
struct Id<M>
where
    M: Model,
{
    /// Simple integer ID which must be unique
    id: usize,
    /// Estimated cost including the heuristic
    f: Reverse<M::Cost>,
    /// Cost to arrive at this node following the parents
    g: M::Cost,
}

impl<M> Id<M>
where
    M: Model,
{
    pub fn new(id: usize, f: M::Cost, g: M::Cost) -> Self {
        Id { id, f: Reverse(f), g }
    }

    #[inline(always)]
    pub fn g(&self) -> M::Cost {
        self.g.clone()
    }

    #[inline(always)]
    pub fn f(&self) -> M::Cost {
        self.f.0.clone()
    }
}

impl<M> Clone for Id<M>
where
    M: Model,
{
    fn clone(&self) -> Self {
        Id { id: self.id, f: self.f.clone(), g: self.g.clone() }
    }
}

impl<M> Hash for Id<M>
where
    M: Model,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl<M> PartialEq for Id<M>
where
    M: Model,
{
    fn eq(&self, other: &Self) -> bool {
        self.f == other.f
    }
}

impl<M> Eq for Id<M> where M: Model {}

impl<M> PartialOrd for Id<M>
where
    M: Model,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.f.cmp(&other.f))
    }
}

impl<M> Ord for Id<M>
where
    M: Model,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.f.cmp(&other.f)
    }
}

impl<M> Debug for Id<M>
where
    M: Model,
    M::Cost: Debug,
{
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), std::fmt::Error> {
        fmt.debug_struct("Id")
            .field("g", &self.g)
            .field("f", &self.f)
            .field("id", &self.id)
            .finish()
    }
}

/// Nodes stored for planning
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

impl<M> PartialEq for Node<M>
where
    M: Model,
{
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<M> Eq for Node<M> where M: Model {}

impl<M> PartialOrd for Node<M>
where
    M: Model,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.id.partial_cmp(&other.id)
    }
}

impl<M> Ord for Node<M>
where
    M: Model,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}

impl<M> Debug for Node<M>
where
    M: Model,
    M::Cost: Debug,
    M::State: Debug,
    M::Control: Debug,
{
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), std::fmt::Error> {
        fmt.debug_struct("Node")
            .field("id", &self.id.id)
            .field("g", &self.id.g)
            .field("f", &self.id.f)
            .field("state", &self.state)
            .field("control", &self.control)
            .finish()
    }
}
