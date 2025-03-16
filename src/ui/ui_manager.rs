//! This example illustrates how to create UI text and update it in a system.
//!
//! It displays the current FPS in the top left corner, as well as text that changes color
//! in the bottom right. For text within a scene, please see the text2d example.

use bevy::{
    color::palettes::css::GOLD,
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};

use crate::utils::resource;

// A unit struct to help identify the FPS UI component, since there may be many Text components
#[derive(Component)]
struct FpsText;

// A unit struct to help identify the color-changing Text component
#[derive(Component)]
struct PlayerScoreText;

#[derive(Component)]
struct AIScoreText;

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        // app.add_systems(Startup, Background::setup);
        app.add_plugins(FrameTimeDiagnosticsPlugin);
        app.add_systems(Startup, setup);
        app.add_systems(
            Update,
            (text_update_system, text_color_system, ai_text_score_system),
        );
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // UI camera
    // commands.spawn(Camera2d);
    // Text with one section
    commands
        .spawn((
            // Accepts a `String` or any type that converts into a `String`, such as `&str`
            Text::new("Player Score: "),
            TextFont {
                // This font is loaded and will be used instead of the default font.
                font: asset_server.load("ui/font/Bangers-Regular.ttf"),
                font_size: 67.0,
                ..default()
            },
            // Set the justification of the Text
            TextLayout::new_with_justify(JustifyText::Center),
            // Set the style of the Node itself.
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(150.0),
                left: Val::Percent(20.0),
                align_content: AlignContent::Center,
                ..Default::default()
            },
        ))
        .with_child((
            TextSpan::default(),
            // "default_font" feature is unavailable, load a font to use instead.
            TextFont {
                font: asset_server.load("ui/font/Bangers-Regular.ttf"),
                font_size: 67.0,
                ..Default::default()
            },
            TextColor(GOLD.into()),
            PlayerScoreText,
        ));

    commands
        .spawn((
            // Accepts a `String` or any type that converts into a `String`, such as `&str`
            Text::new("AI Score: "),
            TextFont {
                // This font is loaded and will be used instead of the default font.
                font: asset_server.load("ui/font/Bangers-Regular.ttf"),
                font_size: 67.0,
                ..default()
            },
            // Set the justification of the Text
            TextLayout::new_with_justify(JustifyText::Center),
            // Set the style of the Node itself.
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(150.0),
                right: Val::Percent(20.0),
                align_content: AlignContent::Center,
                ..Default::default()
            },
        ))
        .with_child((
            TextSpan::default(),
            // "default_font" feature is unavailable, load a font to use instead.
            TextFont {
                font: asset_server.load("ui/font/Bangers-Regular.ttf"),
                font_size: 67.0,
                ..Default::default()
            },
            TextColor(GOLD.into()),
            AIScoreText,
        ));

    // Text with multiple sections
    commands
        .spawn((
            // Create a Text with multiple child spans.
            Text::new("FPS: "),
            TextFont {
                // This font is loaded and will be used instead of the default font.
                font: asset_server.load("ui/font/Bangers-Regular.ttf"),
                font_size: 42.0,
                ..default()
            },
        ))
        .with_child((
            TextSpan::default(),
            // "default_font" feature is unavailable, load a font to use instead.
            TextFont {
                font: asset_server.load("ui/font/Bangers-Regular.ttf"),
                font_size: 33.0,
                ..Default::default()
            },
            TextColor(GOLD.into()),
            FpsText,
        ));

    // commands.spawn((
    //     Text::new("Default font disabled"),
    //     TextFont {
    //         font: asset_server.load("ui/font/Bangers-Regular.ttf"),
    //         ..default()
    //     },
    //     Node {
    //         position_type: PositionType::Absolute,
    //         bottom: Val::Px(5.0),
    //         left: Val::Px(15.0),
    //         ..default()
    //     },
    // ));
}

fn text_color_system(
    // time: Res<Time>,
    player_score: Res<resource::PlayerScore>,
    mut query: Query<&mut TextSpan, With<PlayerScoreText>>,
) {
    for mut text in &mut query {
        // Update the color of the ColorText span.
        **text = format!("{} ", player_score.0);
    }
}

fn ai_text_score_system(
    // time: Res<Time>,
    ai_score: Res<resource::AIScore>,
    mut query: Query<&mut TextSpan, With<AIScoreText>>,
) {
    for mut text in &mut query {
        // Update the color of the ColorText span.
        **text = format!("{} ", ai_score.0);
    }
}

fn text_update_system(
    diagnostics: Res<DiagnosticsStore>,
    mut query: Query<&mut TextSpan, With<FpsText>>,
) {
    for mut span in &mut query {
        if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(value) = fps.smoothed() {
                // Update the value of the second section
                **span = format!("{value:.2}");
            }
        }
    }
}
