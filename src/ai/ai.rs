use bevy::prelude::*;

use crate::{
    grid::base_grid::Grid,
    utils::{
        constant::{self, GRID_HEIGHT, GRID_WIDTH},
        event::SwapPiecesEvent,
        resource,
    },
};

pub struct AI;

impl AI {
    fn swap(mut grid: Grid, row1: i32, col1: i32, row2: i32, col2: i32) -> Grid {
        if row1 < GRID_WIDTH && row2 < GRID_WIDTH && col1 < GRID_HEIGHT && col2 < GRID_HEIGHT {
            if let (Some(piece1), Some(piece2)) = (
                grid.cell[row1 as usize][col1 as usize],
                grid.cell[row2 as usize][col2 as usize],
            ) {
                grid.cell[row1 as usize][col1 as usize] = Some(piece2);
                grid.cell[row2 as usize][col2 as usize] = Some(piece1);
            }
        }
        return grid;
    }

    fn is_match(grid: Grid, row: i32, col: i32) -> bool {
        if let Some(piece) = grid.cell[row as usize][col as usize] {
            let color = piece.color;
            let next_col = col + 1;
            let prev_col = col - 1;
            let prev_row = row - 1;
            let next_row = row + 1;

            if col > 1 && next_col < GRID_HEIGHT {
                if let (Some(p1), Some(p2)) = (
                    grid.cell[row as usize][prev_col as usize],
                    grid.cell[row as usize][next_col as usize],
                ) {
                    if p1.color == color && p2.color == color {
                        return true;
                    }
                }
            }
            if row > 1 && next_row < GRID_WIDTH {
                if let (Some(p1), Some(p2)) = (
                    grid.cell[prev_row as usize][col as usize],
                    grid.cell[next_row as usize][col as usize],
                ) {
                    if p1.color == color && p2.color == color {
                        return true;
                    }
                }
            }
        }

        return false;
    }

    pub fn find_possible_match(
        mut grid_query: Query<&mut Grid>,
        time: Res<Time>,
        mut timer: ResMut<resource::AIMoveTimer>,
        mut ai_moves: ResMut<resource::AIMoveCount>,
        mut ai_score: ResMut<resource::AIScore>,
        mut ev_swap_piece: EventWriter<SwapPiecesEvent>,
    ) {
        if timer.0.tick(time.delta()).just_finished() {
            let grid = grid_query.single_mut();
            for row in 0..(GRID_WIDTH - 1) {
                for col in 0..(GRID_HEIGHT - 1) {
                    let cloned_grid = grid.clone();
                    let grid_col = AI::swap(cloned_grid, row, col, row, col + 1);
                    if AI::is_match(grid_col.clone(), row, col)
                        || AI::is_match(grid_col.clone(), row, col + 1)
                    {
                        //trigger swap event
                        let dir = Vec2::new((col + 1) as f32, row as f32)
                            - Vec2::new(col as f32, row as f32);
                        ev_swap_piece.send(SwapPiecesEvent {
                            row,
                            column: col,
                            direction: dir,
                        });
                        ai_moves.0 += 1;
                        ai_score.0 += 1;
                        return;
                    }
                    let grid_row = AI::swap(grid.clone(), row, col, row + 1, col);
                    if AI::is_match(grid_row.clone(), row, col)
                        || AI::is_match(grid_row.clone(), row + 1, col)
                    {
                        let dir = Vec2::new(col as f32, (row + 1) as f32)
                            - Vec2::new(col as f32, row as f32);
                        println!("{:?}", dir);
                        ev_swap_piece.send(SwapPiecesEvent {
                            row,
                            column: col,
                            direction: dir,
                        });
                        ai_score.0 += 1;
                        ai_moves.0 += 1;
                        return;
                    }
                }
            }
        }
    }
}
