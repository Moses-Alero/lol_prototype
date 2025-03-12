use bevy::prelude::*;
use grid::base_grid::Grid;
use piece::base_piece::BasePiece;
use utils::{event, resource};

pub mod grid;
pub mod piece;
pub mod utils;

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
        .add_event::<event::SwapPiecesEvent>()
        .add_event::<event::SwapBackEvent>()
        //resources
        .insert_resource(resource::CollapseTimer(Timer::from_seconds(
            1.0,
            TimerMode::Repeating,
        )))
        .insert_resource(resource::DestroyPieceTimer(Timer::from_seconds(
            0.8,
            TimerMode::Repeating,
        )))
        .insert_resource(resource::RefillColumnTimer(Timer::from_seconds(
            1.4,
            TimerMode::Repeating,
        )))
        .insert_resource(resource::SwapBackTimer(Timer::from_seconds(
            0.2,
            TimerMode::Repeating,
        )))
        .init_resource::<resource::SwapBackInfo>()
        .init_resource::<resource::PieceController>()
        .init_resource::<resource::Touch>()
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
