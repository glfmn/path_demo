//! Pathfinding using a specliaized pathfinding algorithms which separate controls from state
//!
//! # States versus Controls
//!
//! A state is how the entity we plan for exists in the game space.  A control is an action we can
//! apply to a state in order to change it.
//!
//! # Using a Model
//!
//! The [`Model`] defines the problem by determining the States and Controls.  More specifically,
//! it defines:
//!
//! - the the model outputs; the planning space
//! - the inputs which must be sampled
//! - how to evalute and estimate the cost of actions
//! - when sampled actions are valid or invalid
//! - when we have reached our goal and can terminate planning
//!
//! Returning to the example above, we can let
//!
//! # Optimizer
//!
//! The optimizer is actually responsible for creating the trajectory, using the model to solve the
//! problem.
//!
//! [`Model`]: /path/trait.Model.html

use std::fmt::Debug;
use std::hash::Hash;
use std::ops::Add;

pub mod astar;

/// Marker trait which is required for the type which a [`Model`] uses to represent costs.
///
/// All algorithms used keep a priority queue which must be sorted by the cost.
///
/// [`Model`]: /path/trait.Model.html
pub trait Cost: Ord + Eq + Default + Add<Output = Self>
where
    Self: Sized,
{
}

impl Cost for usize {}
impl Cost for u8 {}
impl Cost for u16 {}
impl Cost for u32 {}
impl Cost for u64 {}
impl Cost for isize {}
impl Cost for i8 {}
impl Cost for i16 {}
impl Cost for i32 {}
impl Cost for i64 {}

pub trait State {
    type Position: Eq + Hash + Debug;

    fn grid_position(&self) -> Self::Position;
}

/// Interface which defines the problem
///
/// The model defines how costs are estimated and calculated, the mapping between controls and
/// states, and the validity and termination conditions of our problem.
pub trait Model: Clone {
    /// The state of the system as a result of actions taken
    type State: Debug + Clone + State;

    /// Actions which can be taken to effect the system
    type Control: Debug + Clone + Default;

    /// A measurement of the cost in the system
    ///
    /// Note: the cost must implement `Ord` and `Eq` in order for the model to be compatible with
    /// the [`Optimizer`] trait.
    ///
    /// [`Optimizer`]: /path/trait.Optimizer.html
    type Cost: Debug + Clone + Cost;

    /// Determine the cost between two states
    ///
    /// Given a current state and future state, to find the optimal path in terms of a particular
    /// state parameter or state parameters, the cost function provides a method to quantify and
    /// _compare_ different paths, chosing the path that results in the lowest overall cost in
    /// terms of this cost function.
    ///
    /// The canonical cost function for many path-finding applications is simply the euclidian
    /// distance (or some other type of distance calculation), which results in a distance
    /// optimizing model.  However, SBMPO exposes methods to do optimization over arbitrary state
    ///  paramters by using the model, such as:
    ///
    /// - energy consumption
    /// - traversal time
    /// - elevation change
    /// - dollars spent
    fn cost(&self, current: &Self::State, next: &Self::State) -> Self::Cost;

    /// Read and set initial conditions
    ///
    /// Called once planning starts, giving the model access to the first state to perform a
    /// sometimes necessary initialization step, such as:
    ///
    /// - removing obstacles close to the initial state
    /// - calculating extra initial values for the model that depend on the values of the first
    ///   state
    fn init(&mut self, initial: &Self::State);

    /// Termination or convergence condition testing
    ///
    /// Test the current State against the goal to determine if it meets the
    /// convergence criteron against the goal State.
    ///
    /// The `current` [`State`] is tested against for convergence.
    /// The `goal` [`State`] represents the solution.
    /// Returns `true` when a solution is found
    ///
    /// [`State`]: #associatedtype.State
    fn converge(&self, current: &Self::State, goal: &Self::State) -> bool;

    /// Generate a new current state from a control which is applied to a previous state
    ///
    /// Since States are not generated directly and expand from previous States, a function is
    /// necessary which maps from previous states to new states according to the generated control.
    ///
    /// In other words, this method defines how states propagate or "integrate" from previous
    /// states according to the controls specified by the model.
    ///
    /// For example, if we plan with position as our States, but control the system in terms of
    /// velocity, we must have a function to apply the velocity to an existing position to get
    /// the next position.
    ///
    /// If the control cannot be applied to the state to produce a valid result, then return
    /// `None`.  This allows for validation like checking to see if the action would collide with
    /// an obstacle.
    fn integrate(
        &self,
        previous: &Self::State,
        control: &Self::Control,
    ) -> Option<Self::State>;
}

/// Heuristic Models are models which can estimate the cost to the goal
pub trait HeuristicModel: Model {
    /// Estimate of future costs from the current state
    ///
    /// - `current` the state to traverse from
    /// - `goal` the overall goal node to estimate the future costs from
    ///
    /// Given the current state and the goal state, what can we estimate the future costs will be?
    /// The heuristic determines where the most fertile paths to search exist, assuming that
    /// continuing along a direct path to the goal will result in the most efficient overall
    /// solution.  This ensures that paths which take a less direct route are explored last.
    ///
    /// The canonical heuristic function is often also the euclidian distance from the current
    /// state to the goal state.
    ///
    /// The heuristic works best when its units are the same--or at least in the same order of
    /// magnitude--as the cost.
    ///
    /// \warning The heuristic must be admissable or optimistic to get optimal results; that is,
    /// the heuristic **must never over-estimate the cost** of a future path.  Over-estimation
    /// breaks optimality guarantees. Furthermore the heuristic must never return a negative value.
    fn heuristic(&self, current: &Self::State, goal: &Self::State) -> Self::Cost;
}

pub trait Sampler<M>
where
    M: Model,
{
    fn sample(&mut self, model: &M, current: &M::State) -> &[M::Control];
}

/// The result of optimization: a trajectory from the start to goal
///
/// A trajectory which carries the cost of its execution, and all of the steps as pairs of states
/// and controls, who's types are determined by the Model.
#[derive(Debug, Clone, PartialEq)]
pub struct Trajectory<M>
where
    M: Model,
{
    pub cost: M::Cost,
    pub trajectory: Vec<(M::State, M::Control)>,
}

impl<M> Default for Trajectory<M>
where
    M: Model,
{
    fn default() -> Self {
        Trajectory { cost: Default::default(), trajectory: Vec::new() }
    }
}

/// Errors that result from
#[derive(Debug, Clone, PartialEq)]
pub enum PathFindingErr {
    Unreachable,
    IterationLimit(usize),
}

pub enum PathResult<M>
where
    M: Model,
{
    Final(Trajectory<M>),
    Intermediate(Trajectory<M>),
    Err(PathFindingErr),
}

/// A strategy to find a trajectory from the start state to the goal state
pub trait Optimizer<M, S>
where
    M: Model,
    M::Cost: Ord + Eq + Default,
    S: Sampler<M>,
{
    /// Trajectory to the head node in the planning queue, not to the optimal solution
    fn next_trajectory(
        &mut self,
        model: &mut M,
        start: &M::State,
        goal: &M::State,
        sampler: &mut S,
    ) -> PathResult<M>;

    /// Calcualte an optimal trajectory with SBMPO
    ///
    /// Using the types defiend by the provided model, we find the optimial trajectory which
    /// connects the start and goal states by sampling controls using the states.
    fn optimize(
        &mut self,
        model: &mut M,
        start: &M::State,
        goal: &M::State,
        sampler: &mut S,
    ) -> PathResult<M>;
}
