use super::Position;

pub trait Action {
    fn execute(&self);
}

pub trait Actor {
    fn take_turn(&self) -> Action;
}
