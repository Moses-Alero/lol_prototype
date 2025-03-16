use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct SwapBackTimer(pub Timer);

#[derive(Resource, Default, Debug)]
pub struct SwapBackInfo {
    pub p1: Vec2,
    pub p2: Vec2,
    pub dir: Vec2,
    pub count: i32,
}

#[derive(Resource, Default)]
pub struct PieceController {
    pub controlling: bool,
}

#[derive(Resource, Default)]
pub struct PlayerMoveCount(pub i32);

#[derive(Resource, Default)]
pub struct AIMoveCount(pub i32);

#[derive(Resource, Default)]
pub struct PlayerScore(pub i32);

#[derive(Resource, Default)]
pub struct AIScore(pub i32);

#[derive(Resource, Default)]
pub struct RefillColumnTimer(pub Timer);

#[derive(Resource, Default)]
pub struct Touch {
    pub first: Vec2,
    pub last: Vec2,
}

#[derive(Resource)]
pub struct CollapseTimer(pub Timer);

#[derive(Resource, Default)]
pub struct DestroyPieceTimer(pub Timer);

#[derive(Resource, Default)]
pub struct AIMoveTimer(pub Timer);

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum CurrentPlayerTurn {
    #[default]
    Player,
    AI,
}
