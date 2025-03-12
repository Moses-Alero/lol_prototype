use crate::grid::base_grid::Grid;
use crate::utils::{constant, event, resource};
use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Eq, PartialEq, Hash, Clone, Copy, Debug)]
pub enum BaseColor {
    Blue,
    Green,
    Yellow,
    Pink,
}

#[derive(Component, Debug, Clone, Copy)]
#[require(Sprite, Transform)]
pub struct BasePiece {
    pub row: i32,
    pub col: i32,
    pub color: BaseColor,
    pub matched: bool,
}

impl BasePiece {
    pub const BASE_COLORS: &[BaseColor] = &[
        BaseColor::Blue,
        BaseColor::Pink,
        BaseColor::Green,
        BaseColor::Yellow,
    ];

    pub fn set_color_path() -> HashMap<BaseColor, String> {
        let mut base_color_path: HashMap<BaseColor, String> = HashMap::new();

        base_color_path.insert(BaseColor::Blue, String::from("pieces/blue_piece.png"));
        base_color_path.insert(BaseColor::Green, String::from("pieces/green_piece.png"));
        base_color_path.insert(BaseColor::Yellow, String::from("pieces/yellow_piece.png"));
        base_color_path.insert(BaseColor::Pink, String::from("pieces/pink_piece.png"));

        return base_color_path;
    }

    pub fn new(row: i32, col: i32, color: BaseColor) -> BasePiece {
        BasePiece {
            row,
            col,
            color,
            matched: false,
        }
    }

    pub fn update_piece_positions(mut piece_query: Query<(&BasePiece, &mut Transform)>) {
        for (piece, mut transform) in piece_query.iter_mut() {
            let target_position = Vec3::new(
                constant::GRID_POSITION.x + (piece.col as f32 * constant::CELL_SIZE),
                constant::GRID_POSITION.y + (piece.row as f32 * constant::CELL_SIZE),
                transform.translation.z,
            );

            // For smooth animation:
            transform.translation = transform.translation.lerp(target_position, 0.2);
        }
    }

    pub fn highlight_match(mut piece_query: Query<(&BasePiece, &mut Sprite)>) {
        for (piece, mut sprite) in piece_query.iter_mut() {
            if piece.matched {
                let new_sprite_color = Color::srgb(0.8, 0.8, 0.0);
                sprite.color = new_sprite_color;
            }
        }
    }

    pub fn destroy_match(
        mut commands: Commands,
        time: Res<Time>,
        mut timer: ResMut<resource::DestroyPieceTimer>,
        mut swap_back: ResMut<resource::SwapBackInfo>,
        mut ev_swap_back: EventWriter<event::SwapBackEvent>,
        mut grid_query: Query<&mut Grid>,
    ) {
        if timer.0.tick(time.delta()).just_finished() {
            let mut grid = grid_query.single_mut();
            let mut match_found = false;
            for row in 0..grid.width {
                for col in 0..grid.height {
                    if let Some(piece) = grid.cell[row as usize][col as usize] {
                        if let Some(piece_entity) = grid.entities[row as usize][col as usize] {
                            if piece.matched {
                                match_found = true;
                                swap_back.count = 0;
                                commands.entity(piece_entity).despawn();
                                grid.entities[row as usize][col as usize] = None;
                                grid.cell[row as usize][col as usize] = None;
                            }
                        }
                    }
                }
            }
            if !match_found && swap_back.count == 1 {
                ev_swap_back.send(event::SwapBackEvent {
                    row: swap_back.p2.y as i32,
                    column: swap_back.p2.x as i32,
                    direction: swap_back.dir * -1.0,
                });
            }
        }
    }
    // fn grid_to_position(&mut self, row: f32, column: f32) {
    //     self.position.x = GRID_POSITION.x + (row * CELL_SIZE);
    //     self.position.y = GRID_POSITION.y + (column * CELL_SIZE);
    // }
}
