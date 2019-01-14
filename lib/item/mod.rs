struct Item<A, D, P, U>
where
    A: Attack,
    D: Defense,
    P: Effect,
    D: Effect,
{
    attack: Option<A>,
    defense: Option<D>,
    passive: Option<P>,
    use_effect: Option<U>,
}

trait Effect {}

trait Attack {}

trait Defense {}
