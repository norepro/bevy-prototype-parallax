use bevy::prelude::*;
use bevy_prototype_parallax::{Layer, LayerBundle, ParallaxPlugin, WindowSize};
use bevy_spritesheet_animation::{
    plugin::SpritesheetAnimationPlugin,
    prelude::{Animation, AnimationDuration, Spritesheet, SpritesheetAnimation},
};

const PLAYER_SPEED: f32 = 1600.0;

#[derive(Component)]
struct Player {
    run_image: Handle<Image>,
    run_layout: Handle<Animation>,
    idle_image: Handle<Image>,
    idle_layout: Handle<Animation>,
}

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Forest".to_string(),
                        resolution: bevy::window::WindowResolution::new(1280, 720),
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_systems(Startup, (setup_parallax, setup_character))
        .add_systems(Update, (move_character_system, follow_player_camera))
        .add_plugins(SpritesheetAnimationPlugin)
        .add_plugins(ParallaxPlugin)
        .run();
}

/// Set up our background layers
fn setup_parallax(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Helper that loads an asset as a parallax layer
    // layers should have different speeds to achieve the effect
    // z controls front-to-back ordering (higher z = more in front)
    let layer = |path: &'static str, speed: f32, z: f32| -> LayerBundle {
        let image = asset_server.load(path);
        LayerBundle {
            layer: Layer { speed, image },
            transform: Transform {
                scale: Vec3::new(4.0, 4.5, 1.0),
                translation: Vec3::new(0.0, 0.0, z),
                ..Default::default()
            },
            ..Default::default()
        }
    };

    // Note the backgrounds are associated with a camera.
    commands
        .spawn((Camera2d, WindowSize::default()))
        .with_children(|cb| {
            // Spawn the layers.
            // We can have as many as we like
            cb.spawn(layer("parallax-forest-back-trees.png", 0.0, 0.0));
            cb.spawn(layer("parallax-forest-lights.png", 0.05, 0.1));
            cb.spawn(layer("parallax-forest-middle-trees.png", 0.1, 0.2));
            cb.spawn(layer("parallax-forest-front-trees.png", 0.2, 0.3));
        });
}

/// Spawns our character and loads it's resources
fn setup_character(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut animations: ResMut<Assets<Animation>>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let run_image: Handle<Image> = asset_server.load("Run.png");
    let run_spritesheet = Spritesheet::new(&run_image, 8, 1);
    let run_animation = run_spritesheet
        .create_animation()
        .add_row(0)
        .set_duration(AnimationDuration::PerFrame(100))
        .build();

    let run_layout = animations.add(run_animation);

    let idle_image: Handle<Image> = asset_server.load("Idle.png");
    let spritesheet = Spritesheet::new(&idle_image, 8, 1);
    let idle_animation = spritesheet
        .create_animation()
        .add_row(0)
        .set_duration(AnimationDuration::PerFrame(100))
        .build();

    let idle_layout = animations.add(idle_animation);

    let sprite = spritesheet
        .with_size_hint(1200, 150)
        .sprite(&mut texture_atlas_layouts);

    commands.spawn((
        sprite,
        SpritesheetAnimation::new(idle_layout.clone()),
        Transform {
            scale: Vec3::new(5.0, 5.0, 1.0),
            translation: Vec3::new(0.0, -220.0, 1.0),
            ..Default::default()
        },
        Player {
            run_image,
            run_layout,
            idle_image,
            idle_layout,
        },
    ));
}

/// Moves the character and sets the appropriate atlas for animation
fn move_character_system(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    query: Single<(
        &Player,
        &mut Sprite,
        &mut SpritesheetAnimation,
        &mut Transform,
    )>,
) {
    let (player, mut sprite, mut animation, mut transform) = query.into_inner();

    if keyboard_input.pressed(KeyCode::KeyA) || keyboard_input.pressed(KeyCode::KeyD) {
        if animation.animation != player.run_layout {
            animation.switch(player.run_layout.clone());
            sprite.image = player.run_image.clone();
        }

        let translation = PLAYER_SPEED * time.delta_secs();

        if keyboard_input.pressed(KeyCode::KeyA) {
            transform.translation.x -= translation;
            sprite.flip_x = true;
        } else {
            transform.translation.x += translation;
            sprite.flip_x = false;
        }
    } else {
        if animation.animation != player.idle_layout {
            animation.switch(player.idle_layout.clone());
            sprite.image = player.idle_image.clone();
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
