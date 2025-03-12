use bevy::prelude::*;

#[derive(Event)]
pub struct SwapBackEvent {
    pub row: i32,
    pub column: i32,
    pub direction: Vec2,
}

#[derive(Event)]
pub struct SwapPiecesEvent {
    pub row: i32,
    pub column: i32,
    pub direction: Vec2,
}
