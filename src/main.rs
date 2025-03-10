use bevy::{prelude::*, window::PrimaryWindow};
use rand::Rng;
use std::collections::HashMap;

#[derive(Eq, PartialEq, Hash, Clone, Copy, Debug)]
enum BaseColor {
    Blue,
    Green,
    Yellow,
    Pink,
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
enum GamePlaySet {
    Input,
    GridLogic,
    // VisualUpdate,
    SwapBack,
    MatchDetection,
}

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
enum SwapBackState {
    // Swap,
    #[default]
    Block,
}

#[derive(Resource, Default)]
struct SwapBackTimer(Timer);

#[derive(Resource, Default, Debug)]
struct SwapBackInfo {
    p1: Vec2,
    p2: Vec2,
    dir: Vec2,
    count: i32,
}

#[derive(Event)]
struct SwapBackEvent {
    row: i32,
    column: i32,
    direction: Vec2,
}

#[derive(Event)]
struct SwapPiecesEvent {
    row: i32,
    column: i32,
    direction: Vec2,
}

const CELL_SIZE: f32 = 50.0;
const GRID_WIDTH: i32 = 7;
const GRID_HEIGHT: i32 = 7;
const GRID_POSITION: Vec3 = Vec3::new(-150.0, -190.0, 1.0);

fn remove_index<T: Clone>(arr: &[T], index: usize) -> Vec<T> {
    let (head, tail) = arr.split_at(index);
    // `tail` starts at the index we want to remove, so we skip the first element of `tail`
    let mut new_vec = head.to_vec();
    new_vec.extend_from_slice(&tail[1..]);
    new_vec
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, base_setup)
        .add_systems(Startup, Grid::setup_grid)
        .configure_sets(
            Startup,
            (
                GamePlaySet::Input,
                GamePlaySet::GridLogic,
                GamePlaySet::MatchDetection,
            )
                .chain(),
        )
        .add_systems(
            Update,
            (
                Grid::mouse_input,
                Grid::swap_pieces,
                BasePiece::update_piece_positions,
            )
                .chain()
                .in_set(GamePlaySet::Input),
        )
        .add_systems(
            Update,
            (
                Grid::find_match,
                // BasePiece::highlight_match,/
                BasePiece::destroy_match,
            )
                .chain()
                .in_set(GamePlaySet::MatchDetection)
                .before(GamePlaySet::SwapBack)
                .after(GamePlaySet::Input),
        )
        .add_systems(
            Update,
            (
                BasePiece::highlight_match,
                Grid::collapse_column,
                Grid::refill_columns,
            )
                .chain()
                .in_set(GamePlaySet::GridLogic)
                .after(GamePlaySet::MatchDetection),
        )
        .add_systems(Update, Grid::swap_back)
        //plugin
        .add_plugins(BackgroundPlugin)
        //events
        .add_event::<SwapPiecesEvent>()
        .add_event::<SwapBackEvent>()
        //resources
        .insert_resource(CollapseTimer(Timer::from_seconds(
            1.0,
            TimerMode::Repeating,
        )))
        .insert_resource(DestroyPieceTimer(Timer::from_seconds(
            0.8,
            TimerMode::Repeating,
        )))
        .insert_resource(RefillColumnTimer(Timer::from_seconds(
            1.4,
            TimerMode::Repeating,
        )))
        .insert_resource(SwapBackTimer(Timer::from_seconds(
            0.2,
            TimerMode::Repeating,
        )))
        .init_resource::<SwapBackInfo>()
        .init_resource::<PieceController>()
        .init_resource::<Touch>()
        //state
        .init_state::<SwapBackState>()
        .run();
}

fn base_setup(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Transform {
            translation: Vec3::new(0.0, 0.0, 0.0),
            ..default()
        },
    ));
}
#[derive(Resource, Default)]
struct PieceController {
    controlling: bool,
}

#[derive(Component)]
#[require(Sprite)]
struct Background;

pub struct BackgroundPlugin;

impl Plugin for BackgroundPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, Background::setup);
    }
}

impl Background {
    fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
        commands.spawn((
            Background,
            Sprite::from_image(asset_server.load("ui/backgrounds/background1.png")),
            Transform {
                translation: Vec3::new(0.0, 0.0, 0.0),
                ..Default::default()
            },
        ));
    }
}

#[derive(Component, Debug, Clone)]
struct Grid {
    width: i32,
    height: i32,
    cell: Vec<Vec<Option<BasePiece>>>,
    entities: Vec<Vec<Option<Entity>>>,
}

#[derive(Resource, Default)]
struct RefillColumnTimer(Timer);

#[derive(Resource, Default)]
struct Touch {
    first: Vec2,
    last: Vec2,
}

#[derive(Resource)]
struct CollapseTimer(Timer);

impl Grid {
    fn new(width: i32, height: i32) -> Grid {
        Grid {
            width,
            height,
            cell: Vec::with_capacity(height as usize),
            entities: vec![vec![None; GRID_WIDTH as usize]; GRID_HEIGHT as usize],
        }
    }

    fn setup_grid(mut commands: Commands, asset_server: Res<AssetServer>) {
        let mut grid = Grid::new(GRID_WIDTH, GRID_HEIGHT);
        let mut rng = rand::rng();
        let base_piece_color_path = BasePiece::set_color_path();
        let color_set = [(0.0, 0.2, 0.0), (0.2, 0.4, 0.2)];
        let mut color_choice = color_set[0];
        for row in 0..grid.width {
            let mut row_entities: Vec<Option<BasePiece>> = Vec::with_capacity(grid.width as usize);
            for col in 0..grid.height {
                let mut rand = rng.random_range(..BasePiece::BASE_COLORS.len());
                let mut base_color = BasePiece::BASE_COLORS[rand];

                if Grid::match_col_at(&mut row_entities, col, base_color) {
                    let new_base_colors = remove_index(BasePiece::BASE_COLORS, rand);
                    rand = rng.random_range(..new_base_colors.len());
                    base_color = new_base_colors[rand];
                }
                if grid.match_at(row, col, base_color) {
                    let new_base_colors = remove_index(BasePiece::BASE_COLORS, rand);
                    rand = rng.random_range(..new_base_colors.len());
                    base_color = new_base_colors[rand];
                }
                let piece = BasePiece::new(row, col, base_color);
                row_entities.push(Some(piece));
                let path = base_piece_color_path.get(&piece.color).unwrap();
                let piece_path: Handle<Image> = asset_server.load(path);
                let piece_position = Vec3::new(
                    GRID_POSITION.x + (piece.col as f32 * CELL_SIZE),
                    GRID_POSITION.y + (piece.row as f32 * CELL_SIZE),
                    3.0,
                );
                let piece_entity_commands = commands.spawn((
                    piece,
                    Transform {
                        translation: piece_position,
                        scale: Vec3::new(0.3, 0.3, 0.),
                        ..Default::default()
                    },
                    Sprite::from_image(piece_path),
                ));
                grid.entities[row as usize][col as usize] = Some(piece_entity_commands.id());

                //alternate and setup grid background
                if color_choice == color_set[0] {
                    color_choice = color_set[1];
                } else {
                    color_choice = color_set[0];
                }

                commands.spawn((
                    Sprite {
                        color: Color::srgb(
                            color_choice.0 as f32,
                            color_choice.0 as f32,
                            color_choice.0 as f32,
                        ),
                        // Specify the width and height of the box.
                        custom_size: Some(Vec2::new(CELL_SIZE, CELL_SIZE)),
                        ..default()
                    },
                    Transform {
                        translation: Vec3::new(
                            GRID_POSITION.x + (piece.col as f32 * CELL_SIZE),
                            GRID_POSITION.y + (piece.row as f32 * CELL_SIZE),
                            2.0,
                        ),
                        ..default()
                    },
                ));
            }
            grid.cell.push(row_entities);
        }
        commands.spawn((
            grid,
            Transform {
                translation: GRID_POSITION,
                ..default()
            },
        ));
    }

    fn match_col_at(entities: &mut Vec<Option<BasePiece>>, col: i32, color: BaseColor) -> bool {
        // Check for horizontal matches (left direction)
        if col > 1 {
            let col1 = col - 1;
            let col2 = col - 2;
            if let (Some(piece1), Some(piece2)) =
                (entities.get(col1 as usize), entities.get(col2 as usize))
            {
                if let (Some(p1), Some(p2)) = (piece1, piece2) {
                    if p1.color == color && p2.color == color {
                        return true;
                    }
                }
            }
        }
        return false;
    }

    fn match_at(&self, row: i32, col: i32, color: BaseColor) -> bool {
        if row > 1 {
            let row1 = row - 1;
            let row2 = row - 2;
            if let (Some(piece1), Some(piece2)) = (
                self.cell
                    .get(row1 as usize)
                    .and_then(|r| r.get(col as usize)),
                self.cell
                    .get(row2 as usize)
                    .and_then(|r| r.get(col as usize)),
            ) {
                if let (Some(p1), Some(p2)) = (piece1, piece2) {
                    if p1.color == color && p2.color == color {
                        return true;
                    }
                }
            }
        }
        return false;
    }

    fn mouse_input(
        mut grid_query: Query<&mut Grid>,
        ev_swap_piece: EventWriter<SwapPiecesEvent>,
        windows: Query<&mut Window, With<PrimaryWindow>>,
        q_camera: Query<(&Camera, &GlobalTransform)>,
        mouse: Res<ButtonInput<MouseButton>>,
        mut controller: ResMut<PieceController>,
        mut touch: ResMut<Touch>,
    ) {
        let mut grid = grid_query.single_mut();
        if let Ok(window) = windows.get_single() {
            let (camera, camera_transform) = q_camera.single();
            {
                if mouse.just_pressed(MouseButton::Left) {
                    let cursor_position = window.cursor_position().unwrap();
                    let ray = camera.viewport_to_world_2d(camera_transform, cursor_position);
                    match ray {
                        Ok(ray_pos) => {
                            let (row, column) = grid.to_grid_position(ray_pos);
                            touch.first = Vec2::new(column as f32, row as f32);
                            //check if the position is in the grid
                            if Grid::is_in_grid(row, column) {
                                controller.controlling = true;
                            }
                        }
                        Err(_) => {}
                    }
                }

                if mouse.just_released(MouseButton::Left) {
                    let cursor_position = window.cursor_position().unwrap();
                    let ray = camera.viewport_to_world_2d(camera_transform, cursor_position);
                    match ray {
                        Ok(ray_pos) => {
                            let (row, column) = grid.to_grid_position(ray_pos);
                            touch.last = Vec2::new(column as f32, row as f32);
                            //check if the position is in the grid
                            if Grid::is_in_grid(row, column) && controller.controlling {
                                grid.touch_diff(ev_swap_piece, touch.first, touch.last);
                                controller.controlling = false;
                            }
                        }
                        Err(_) => {}
                    }
                }
            }
        }
    }

    fn touch_diff(
        &mut self,
        mut ev_swap_piece: EventWriter<SwapPiecesEvent>,
        first: Vec2,
        last: Vec2,
    ) {
        let diff = last - first;
        if diff.y != 0.0 {
            if diff.y > 0.0 {
                ev_swap_piece.send(SwapPiecesEvent {
                    row: first.y as i32,
                    column: first.x as i32,
                    direction: Vec2::new(0.0, 1.0),
                });
            } else if diff.y < 0.0 {
                ev_swap_piece.send(SwapPiecesEvent {
                    row: first.y as i32,
                    column: first.x as i32,
                    direction: Vec2::new(0.0, -1.0),
                });
            }
        } else if diff.x != 0.0 {
            if diff.x > 0.0 {
                ev_swap_piece.send(SwapPiecesEvent {
                    row: first.y as i32,
                    column: first.x as i32,
                    direction: Vec2::new(1.0, 0.0),
                });
            } else if diff.x < 0.0 {
                ev_swap_piece.send(SwapPiecesEvent {
                    row: first.y as i32,
                    column: first.x as i32,
                    direction: Vec2::new(-1.0, 0.0),
                });
            }
        }
    }

    fn is_in_grid(pos_row: i32, pos_column: i32) -> bool {
        if pos_row >= 0 && pos_row < GRID_WIDTH {
            if pos_column >= 0 && pos_column < GRID_HEIGHT {
                return true;
            }
        }
        return false;
    }

    fn to_grid_position(&self, position: Vec2) -> (i32, i32) {
        let pos_row = (position.y - GRID_POSITION.y) / CELL_SIZE;
        let pos_column = (position.x - GRID_POSITION.x) / CELL_SIZE;

        return (pos_row.round() as i32, pos_column.round() as i32);
    }

    fn swap_pieces(
        mut grid_query: Query<&mut Grid>,
        mut ev_swap_piece: EventReader<SwapPiecesEvent>,
        mut swap_back: ResMut<SwapBackInfo>,
        mut commands: Commands,
    ) {
        for ev in ev_swap_piece.read() {
            let mut grid = grid_query.single_mut();
            let pos1 = (ev.row as usize, ev.column as usize);
            let pos2 = (
                (ev.row as f32 + ev.direction.y) as usize,
                (ev.column as f32 + ev.direction.x) as usize,
            );
            if pos1.0 < grid.cell.len()
                && pos1.1 < grid.cell[pos1.0].len()
                && pos2.0 < grid.cell.len()
                && pos2.1 < grid.cell[pos2.0].len()
            {
                if let (Some(p1), Some(p2)) = (grid.cell[pos1.0][pos1.1], grid.cell[pos2.0][pos2.1])
                {
                    let temp_entity = grid.entities[pos1.0][pos1.1];

                    grid.entities[pos1.0][pos1.1] = grid.entities[pos2.0][pos2.1];
                    grid.entities[pos2.0][pos2.1] = temp_entity;

                    // Swap piece data
                    let temp_piece = grid.cell[pos1.0][pos1.1];
                    let temp_row = p1.row;
                    let temp_col = p1.col;

                    let temp_row2 = p2.row;
                    let temp_col2 = p2.col;

                    grid.cell[pos1.0][pos1.1] = grid.cell[pos2.0][pos2.1];

                    grid.cell[pos1.0][pos1.1] = Some(BasePiece {
                        row: temp_row,
                        col: temp_col,
                        color: grid.cell[pos1.0][pos1.1].unwrap().color,
                        matched: p2.matched,
                    });

                    grid.cell[pos2.0][pos2.1] = temp_piece;
                    grid.cell[pos2.0][pos2.1] = Some(BasePiece {
                        row: temp_row2,
                        col: temp_col2,
                        color: grid.cell[pos2.0][pos2.1].unwrap().color,
                        matched: p1.matched,
                    });

                    // Update the BasePiece components with new row/col values
                    if let Some(entity1) = grid.entities[pos1.0][pos1.1] {
                        commands.entity(entity1).insert(BasePiece {
                            row: pos1.0 as i32,
                            col: pos1.1 as i32,
                            color: p1.color,
                            matched: p1.matched,
                        });
                    }

                    if let Some(entity2) = grid.entities[pos2.0][pos2.1] {
                        commands.entity(entity2).insert(BasePiece {
                            row: pos2.0 as i32,
                            col: pos2.1 as i32,
                            color: p2.color,
                            matched: p2.matched,
                        });
                    }
                }

                // send swap info to the swap back resource
                swap_back.p1 = Vec2::new(ev.column as f32, ev.row as f32);
                swap_back.p2 = Vec2::new(pos2.1 as f32, pos2.0 as f32);
                swap_back.dir = ev.direction;
                swap_back.count = 1;
            }
        }
    }

    fn swap_back(
        mut commands: Commands,
        mut ev_swap_back: EventReader<SwapBackEvent>,
        mut swap_back: ResMut<SwapBackInfo>,
        mut grid_query: Query<&mut Grid>,
    ) {
        for ev in ev_swap_back.read() {
            if swap_back.count == 1 {
                let mut grid = grid_query.single_mut();
                let pos1 = (ev.row as usize, ev.column as usize);
                let pos2 = (
                    (ev.row as f32 + ev.direction.y) as usize,
                    (ev.column as f32 + ev.direction.x) as usize,
                );

                if pos1.0 < grid.cell.len()
                    && pos1.1 < grid.cell[pos1.0].len()
                    && pos2.0 < grid.cell.len()
                    && pos2.1 < grid.cell[pos2.0].len()
                {
                    if let (Some(p1), Some(p2)) =
                        (grid.cell[pos1.0][pos1.1], grid.cell[pos2.0][pos2.1])
                    {
                        let temp_entity = grid.entities[pos1.0][pos1.1];

                        grid.entities[pos1.0][pos1.1] = grid.entities[pos2.0][pos2.1];
                        grid.entities[pos2.0][pos2.1] = temp_entity;

                        // Swap piece data
                        let temp_piece = grid.cell[pos1.0][pos1.1];
                        let temp_row = p1.row;
                        let temp_col = p1.col;

                        let temp_row2 = p2.row;
                        let temp_col2 = p2.col;

                        grid.cell[pos1.0][pos1.1] = grid.cell[pos2.0][pos2.1];

                        grid.cell[pos1.0][pos1.1] = Some(BasePiece {
                            row: temp_row,
                            col: temp_col,
                            color: grid.cell[pos1.0][pos1.1].unwrap().color,
                            matched: p2.matched,
                        });

                        grid.cell[pos2.0][pos2.1] = temp_piece;
                        grid.cell[pos2.0][pos2.1] = Some(BasePiece {
                            row: temp_row2,
                            col: temp_col2,
                            color: grid.cell[pos2.0][pos2.1].unwrap().color,
                            matched: p1.matched,
                        });

                        // Update the BasePiece components with new row/col values
                        if let Some(entity1) = grid.entities[pos1.0][pos1.1] {
                            commands.entity(entity1).insert(BasePiece {
                                row: pos1.0 as i32,
                                col: pos1.1 as i32,
                                color: p1.color,
                                matched: p1.matched,
                            });
                        }

                        if let Some(entity2) = grid.entities[pos2.0][pos2.1] {
                            commands.entity(entity2).insert(BasePiece {
                                row: pos2.0 as i32,
                                col: pos2.1 as i32,
                                color: p2.color,
                                matched: p2.matched,
                            });
                        }
                    }
                }
                swap_back.count = 0;
            }
        }
    }

    fn find_match(mut commands: Commands, mut grid_query: Query<&mut Grid>) {
        let mut grid = grid_query.single_mut();
        for row in 0..grid.width {
            for col in 0..grid.height {
                let piece = grid.cell[row as usize][col as usize];
                if grid.entities[row as usize][col as usize].is_none() {
                    continue;
                }
                if let Some(base_piece) = piece {
                    let current_color = base_piece.color;

                    // Horizontal Match
                    if let (Some(col1), Some(col2)) = (
                        (col > 0).then_some(col - 1),
                        (col < grid.height - 1).then_some(col + 1),
                    ) {
                        let piece1 = grid.cell[row as usize][col1 as usize];
                        let piece2 = grid.cell[row as usize][col2 as usize];

                        if let (Some(p1), Some(p2)) = (piece1, piece2) {
                            if p1.color == current_color && p2.color == current_color {
                                for &p in &[p1, base_piece, p2] {
                                    if let Some(entity) =
                                        grid.entities[p.row as usize][p.col as usize]
                                    {
                                        grid.cell[p.row as usize][p.col as usize] =
                                            Some(BasePiece {
                                                row: p.row,
                                                col: p.col,
                                                color: p.color,
                                                matched: true,
                                            });

                                        let e = commands
                                            .entity(entity)
                                            .insert(BasePiece {
                                                row: p.row,
                                                col: p.col,
                                                color: p.color,
                                                matched: true,
                                            })
                                            .id();

                                        grid.entities[p.row as usize][p.col as usize] = Some(e);
                                    }
                                }
                            }
                        }
                    }

                    // Vertical Match
                    if let (Some(row1), Some(row2)) = (
                        (row > 0).then_some(row - 1),
                        (row < grid.width - 1).then_some(row + 1),
                    ) {
                        let piece1 = grid.cell[row1 as usize][col as usize];
                        let piece2 = grid.cell[row2 as usize][col as usize];

                        if let (Some(p1), Some(p2)) = (piece1, piece2) {
                            if p1.color == current_color && p2.color == current_color {
                                for &p in &[p1, base_piece, p2] {
                                    if let Some(entity) =
                                        grid.entities[p.row as usize][p.col as usize]
                                    {
                                        grid.cell[p.row as usize][p.col as usize] =
                                            Some(BasePiece {
                                                row: p.row,
                                                col: p.col,
                                                color: p.color,
                                                matched: true,
                                            });
                                        let e = commands
                                            .entity(entity)
                                            .insert(BasePiece {
                                                row: p.row,
                                                col: p.col,
                                                color: p.color,
                                                matched: true,
                                            })
                                            .id();

                                        grid.entities[p.row as usize][p.col as usize] = Some(e);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn collapse_column(
        mut commands: Commands,
        time: Res<Time>,
        mut timer: ResMut<CollapseTimer>,
        mut grid_query: Query<&mut Grid>,
    ) {
        if timer.0.tick(time.delta()).just_finished() {
            let mut grid = grid_query.single_mut();
            for row in 0..grid.width {
                for col in 0..grid.height {
                    if grid.cell[row as usize][col as usize].is_none() {
                        for k in (row + 1)..grid.width {
                            if let Some(piece) = grid.cell[k as usize][col as usize] {
                                if grid.cell[row as usize][col as usize].is_none() {
                                    grid.entities[row as usize][col as usize] =
                                        grid.entities[k as usize][col as usize];

                                    grid.entities[k as usize][col as usize] = None;

                                    grid.cell[row as usize][col as usize] =
                                        grid.cell[k as usize][col as usize];
                                    grid.cell[row as usize][col as usize] = Some(BasePiece {
                                        row,
                                        col: piece.col,
                                        color: piece.color,
                                        matched: false,
                                    });

                                    grid.cell[k as usize][col as usize] = None;

                                    if let Some(e2) = grid.entities[row as usize][col as usize] {
                                        commands.entity(e2).insert(BasePiece {
                                            row,
                                            col: piece.col,
                                            color: piece.color,
                                            matched: false,
                                        });
                                    }
                                }
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    fn refill_columns(
        mut commands: Commands,
        time: Res<Time>,
        mut timer: ResMut<RefillColumnTimer>,
        mut grid_query: Query<&mut Grid>,
        asset_server: Res<AssetServer>,
    ) {
        let mut grid = grid_query.single_mut();
        if timer.0.tick(time.delta()).just_finished() {
            let mut rng = rand::rng();
            let base_piece_color_path = BasePiece::set_color_path();
            for row in 0..grid.width {
                for col in 0..grid.height {
                    if grid.cell[(GRID_WIDTH - 1) as usize][col as usize].is_none() {
                        if grid.cell[row as usize][col as usize].is_none() {
                            let mut rand = rng.random_range(..BasePiece::BASE_COLORS.len());
                            let mut base_color = BasePiece::BASE_COLORS[rand];

                            if Grid::match_col_at(&mut grid.cell[row as usize], col, base_color) {
                                let new_base_colors = remove_index(BasePiece::BASE_COLORS, rand);
                                rand = rng.random_range(..new_base_colors.len());
                                base_color = new_base_colors[rand];
                            }

                            if grid.match_at(row, col, base_color) {
                                let new_base_colors = remove_index(BasePiece::BASE_COLORS, rand);
                                rand = rng.random_range(..new_base_colors.len());
                                base_color = new_base_colors[rand];
                            }

                            let piece = BasePiece::new(row, col, base_color);
                            grid.cell[row as usize][col as usize] = Some(piece);
                            let path = base_piece_color_path.get(&piece.color).unwrap();
                            let piece_path: Handle<Image> = asset_server.load(path);
                            let piece_position = Vec3::new(
                                GRID_POSITION.x + (col as f32 * CELL_SIZE),
                                GRID_POSITION.y + (GRID_WIDTH as f32 * CELL_SIZE),
                                3.0,
                            );
                            let piece_entity_commands = commands.spawn((
                                piece,
                                Transform {
                                    translation: piece_position,
                                    scale: Vec3::new(0.3, 0.3, 0.),
                                    ..Default::default()
                                },
                                Sprite::from_image(piece_path),
                            ));
                            grid.entities[row as usize][col as usize] =
                                Some(piece_entity_commands.id());
                        }
                    }
                }
            }
        }
    }
}

#[derive(Component, Debug, Clone, Copy)]
#[require(Sprite, Transform)]
struct BasePiece {
    row: i32,
    col: i32,
    color: BaseColor,
    matched: bool,
}
#[derive(Resource, Default)]
struct DestroyPieceTimer(Timer);

impl BasePiece {
    const BASE_COLORS: &[BaseColor] = &[
        BaseColor::Blue,
        BaseColor::Pink,
        BaseColor::Green,
        BaseColor::Yellow,
    ];
    fn set_color_path() -> HashMap<BaseColor, String> {
        let mut base_color_path: HashMap<BaseColor, String> = HashMap::new();

        base_color_path.insert(BaseColor::Blue, String::from("pieces/blue_piece.png"));
        base_color_path.insert(BaseColor::Green, String::from("pieces/green_piece.png"));
        base_color_path.insert(BaseColor::Yellow, String::from("pieces/yellow_piece.png"));
        base_color_path.insert(BaseColor::Pink, String::from("pieces/pink_piece.png"));

        return base_color_path;
    }

    fn new(row: i32, col: i32, color: BaseColor) -> BasePiece {
        BasePiece {
            row,
            col,
            color,
            matched: false,
        }
    }

    fn update_piece_positions(mut piece_query: Query<(&BasePiece, &mut Transform)>) {
        for (piece, mut transform) in piece_query.iter_mut() {
            let target_position = Vec3::new(
                GRID_POSITION.x + (piece.col as f32 * CELL_SIZE),
                GRID_POSITION.y + (piece.row as f32 * CELL_SIZE),
                transform.translation.z,
            );

            // For smooth animation:
            transform.translation = transform.translation.lerp(target_position, 0.2);
        }
    }

    fn highlight_match(mut piece_query: Query<(&BasePiece, &mut Sprite)>) {
        for (piece, mut sprite) in piece_query.iter_mut() {
            if piece.matched {
                let new_sprite_color = Color::srgb(0.8, 0.8, 0.0);
                sprite.color = new_sprite_color;
            }
        }
    }

    fn destroy_match(
        mut commands: Commands,
        time: Res<Time>,
        mut timer: ResMut<DestroyPieceTimer>,
        mut swap_back: ResMut<SwapBackInfo>,
        mut ev_swap_back: EventWriter<SwapBackEvent>,
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
                ev_swap_back.send(SwapBackEvent {
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
