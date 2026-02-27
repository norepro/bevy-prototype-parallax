use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_prototype_parallax::{Layer, LayerBundle, ParallaxPlugin, WindowSize};

#[derive(Component)]
struct Player {
    pub run_image: Handle<Image>,
    pub run_layout: Handle<TextureAtlasLayout>,
    pub idle_image: Handle<Image>,
    pub idle_layout: Handle<TextureAtlasLayout>,
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Forrest".to_string(),
                resolution: bevy::window::WindowResolution::new(1280, 720),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, (setup_parallax, setup_character))
        .add_systems(
            Update,
            (
                move_character_system,
                follow_player_camera,
                animate_sprite_system,
            ),
        )
        .add_plugins(ParallaxPlugin)
        .run();
}

/// Set up our background layers
fn setup_parallax(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Helper that loads an asset as a parallax layer
    // layers should have different speeds to achieve the effect
    let layer = |path: &'static str, speed: f32| -> LayerBundle {
        let image = asset_server.load(path);
        LayerBundle {
            layer: Layer {
                speed,
                ..Default::default()
            },
            sprite: Sprite::from_image(image),
            transform: Transform {
                scale: Vec3::new(4.0, 4.5, 1.0),
                translation: Vec3::new(0.0, 0.0, 0.0),
                ..Default::default()
            },
        }
    };

    // Note the backgrounds are associated with a camera.
    commands
        .spawn((Camera2d, WindowSize::default()))
        .with_children(|cb| {
            // Spawn the layers.
            // We can have as many as we like
            cb.spawn(layer("parallax-forest-back-trees.png", 0.0));
            cb.spawn(layer("parallax-forest-lights.png", 0.05));
            cb.spawn(layer("parallax-forest-middle-trees.png", 0.1));
            cb.spawn(layer("parallax-forest-front-trees.png", 0.2));
        });
}

/// Spawns our character and loads it's resources
fn setup_character(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let run_image = asset_server.load("Run.png");
    let run_layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(24, 24),
        8,
        1,
        None,
        None,
    ));

    let idle_image = asset_server.load("Idle.png");
    let idle_layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(24, 24),
        8,
        1,
        None,
        None,
    ));

    commands.spawn((
        Sprite::from_atlas_image(
            idle_image.clone(),
            TextureAtlas {
                layout: idle_layout.clone(),
                index: 0,
            },
        ),
        Transform {
            scale: Vec3::new(25.0, 25.0, 1.0),
            translation: Vec3::new(0.0, -220.0, 1.0),
            ..Default::default()
        },
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
        Player {
            run_image,
            run_layout,
            idle_image,
            idle_layout,
        },
    ));
}

/// From bevy examples, will animate the sprites in an atlas
fn animate_sprite_system(
    texture_atlas_layouts: Res<Assets<TextureAtlasLayout>>,
    time: Res<Time>,
    mut query: Query<(&mut AnimationTimer, &mut Sprite)>,
) {
    for (mut timer, mut sprite) in query.iter_mut() {
        timer.tick(time.delta());
        if timer.just_finished() {
            if let Some(atlas) = &mut sprite.texture_atlas {
                if let Some(layout) = texture_atlas_layouts.get(&atlas.layout) {
                    atlas.index = (atlas.index + 1) % layout.len();
                }
            }
        }
    }
}

/// Moves the character and sets the appropriate atlas for animation
fn move_character_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&Player, &mut Transform, &mut Sprite)>,
) {
    for (player, mut transform, mut sprite) in query.iter_mut() {
        if keyboard_input.pressed(KeyCode::KeyA) {
            transform.translation.x -= 5.0;
            transform.rotation = Quat::from_rotation_y(PI);
            sprite.image = player.run_image.clone();
            if let Some(atlas) = &mut sprite.texture_atlas {
                atlas.layout = player.run_layout.clone();
            }
        } else if keyboard_input.pressed(KeyCode::KeyD) {
            transform.translation.x += 5.0;
            transform.rotation = Quat::from_rotation_y(0.0);
            sprite.image = player.run_image.clone();
            if let Some(atlas) = &mut sprite.texture_atlas {
                atlas.layout = player.run_layout.clone();
            }
        } else {
            sprite.image = player.idle_image.clone();
            if let Some(atlas) = &mut sprite.texture_atlas {
                atlas.layout = player.idle_layout.clone();
            }
        }
    }
}

/// A simple system that will cause the camera to follow the character
fn follow_player_camera(
    player: Query<&Transform, (With<Player>, Without<Camera2d>)>,
    mut camera: Query<&mut Transform, (With<Camera2d>, Without<Player>)>,
) {
    if let Some(first_player) = player.iter().next() {
        for mut transform in camera.iter_mut() {
            transform.translation.x = first_player.translation.x;
        }
    }
}
