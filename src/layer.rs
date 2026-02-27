use crate::window_size::WindowSize;
use bevy::prelude::*;

#[derive(Component, Default, Debug)]
pub struct Layer {
    pub speed: f32,
    pub image: Handle<Image>,
}

#[derive(Bundle, Default)]
pub struct LayerBundle {
    pub layer: Layer,
    pub transform: Transform,
    pub visibility: Visibility,
}

/// Gets the width of the image asset.
fn get_image_width(image: &Handle<Image>, images: &Assets<Image>) -> f32 {
    images
        .get(image)
        .map(|img| img.size().x as f32)
        .unwrap_or(0.0)
}

/// Gets the 'screen' width of the sprite.
/// This takes into account the scaling
fn sprite_scaled_width(width: f32, transform: &Transform) -> f32 {
    width * transform.scale.x
}

/// Calculate the amount of sprites we need for the effect
fn desired_children_count(window: &WindowSize, width: f32, transform: &Transform) -> f32 {
    let tex_width = sprite_scaled_width(width, transform);
    if tex_width > 0.0 {
        window.width.div_euclid(tex_width) + 2.0
    } else {
        0.0
    }
}

/// Caculates an offset to put the layer at the left edge of the 'container'
/// This is because the camera seems to center on 0.0
fn camera_left_edge_offset(window: &WindowSize) -> f32 {
    let left_side = 0.0 - window.width as f32 / 2.0;
    left_side
}

/// How far to offset the layer due to the camera position
/// Will be clamped by the sprite offset
fn camera_sprite_offset(
    camera: &Vec3,
    layer: &Layer,
    width: f32,
    transform: &Transform,
) -> f32 {
    let sprite_width = sprite_scaled_width(width, transform);
    -(camera.x * layer.speed).rem_euclid(sprite_width)
}

/// Mutates the layer based on the camera position
/// this allows us to have the parallax effect by having the layers move at different rates
/// once we move past the width of the sprite, it resets to 0
fn move_layer_position(
    window: &WindowSize,
    camera: &Vec3,
    width: f32,
    layer: &Layer,
    transform: &mut Transform,
) {
    let offset = camera_left_edge_offset(window);
    let camera_x = camera_sprite_offset(camera, layer, width, transform);

    transform.translation.x = offset + camera_x;
}

/// Manages the amount of child sprites we need to repeat
/// Based on the windows size
pub fn children_count_system(
    mut commands: Commands,
    cameras_query: Query<&WindowSize, With<Camera>>,
    images: Res<Assets<Image>>,
    layer_query: Query<
        (Entity, &ChildOf, Option<&Children>, &Layer, &Transform),
    >,
) {
    for (entity, child_of, children, layer, transform) in layer_query.iter() {
        if let Ok(window) = cameras_query.get(child_of.0) {
            let width = get_image_width(&layer.image, &images);
            let desired_children = desired_children_count(window, width, transform);
            let current_children = children.map(|c| c.len()).unwrap_or(0);
            let to_add = desired_children as usize - current_children;

            for _ in 0..to_add {
                commands.spawn((
                    Sprite::from_image(layer.image.clone()),
                    ChildOf(entity),
                ));
            }

            // TODO: remove sprites if they aren't needed
        }
    }
}

/// Responsible for setting the positioning of the sprites
pub fn children_layout_system(
    images: Res<Assets<Image>>,
    layers: Query<(&Layer, &Children)>,
    mut sprites: Query<&mut Transform>,
) {
    for (layer, children) in layers.iter() {
        let width = get_image_width(&layer.image, &images);
        for (index, child) in children.iter().enumerate() {
            if let Ok(mut transform) = sprites.get_mut(child) {
                transform.translation.x = index as f32 * sprite_scaled_width(width, &transform);
            }
        }
    }
}

/// Matches the layer to the camera.
/// Note the layer is offset to the left by half the window to make
pub fn layer_movement_system(
    images: Res<Assets<Image>>,
    cameras: Query<(&Transform, &WindowSize, &Children), With<Camera>>,
    mut layers: Query<(&Layer, &mut Transform), Without<Camera>>,
) {
    for (transform, window, children) in cameras.iter() {
        let camera = transform.translation;
        for child in children.iter() {
            if let Ok((layer, mut trans)) = layers.get_mut(child) {
                let width = get_image_width(&layer.image, &images);
                move_layer_position(window, &camera, width, layer, &mut trans);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest(
        width,
        expected,
        case(1024.0, -512.0),
        case(1000.0, -500.0)
    )]
    fn test_left_edge(width: f32, expected: f32) {
        let window = WindowSize {
            height: 576.0,
            width: width,
        };
        let result = camera_left_edge_offset(&window);
        assert_eq!(expected, result);
    }

    #[rstest(
        camera,
        speed,
        sprite_width,
        expected,
        case(0.0, 1.0, 100.0, 0.0),
        case(1.0, 1.0, 100.0, -1.0),
        case(101.0, 1.0, 100.0, -1.0),
        case(200.0, 1.0, 100.0, 0.0),
        case(220.0, 1.0, 100.0, -20.0)
        ::trace
    )]
    fn test_layer_offset(camera: f32, speed: f32, sprite_width: f32, expected: f32) {
        let camera = Vec3::new(camera, 0.0, 0.0);
        let transform = Transform::default();
        let layer = Layer { speed, ..Default::default() };
        let result = camera_sprite_offset(&camera, &layer, sprite_width, &transform);
        assert_eq!(expected, result);
    }

    #[rstest(
        sprite_width,
        scale,
        expected,
        case(100.0, 1.0, 100.0),
        case(100.0, 2.0, 200.0),
        case(100.0, 0.0, 0.0),
        case(512.0, 1.0, 512.0)
        ::trace
    )]
    fn test_scaled_width(sprite_width: f32, scale: f32, expected: f32) {
        let transform = Transform {
            scale: Vec3::splat(scale),
            ..Default::default()
        };
        let result = sprite_scaled_width(sprite_width, &transform);
        assert_eq!(expected, result);
    }

    #[rstest(
        screen,
        texture,
        scale,
        expected,
        case(1024.0, 100.0, 1.0, 12.0),
        case(1024.0, 1025.0, 1.0, 2.0),
        case(1024.0, 800.0, 1.0, 3.0),
        case(1024.0, 0.0, 1.0, 0.0)
        ::trace
    )]
    fn test_desired_children_count(screen: f32, texture: f32, scale: f32, expected: f32) {
        let window = WindowSize {
            height: 576.0,
            width: screen,
        };

        let transform = Transform {
            scale: Vec3::splat(scale),
            ..Default::default()
        };

        let result = desired_children_count(&window, texture, &transform);
        assert_eq!(expected, result);
    }

    #[rstest(
        screen, camera, sprite_width, speed, expected,
        case(1024.0, 0.0, 512.0, 0.0, -512.0),
        case(1024.0, 1.0, 512.0, 0.0, -512.0),
        case(1024.0, 512.0, 512.0, 1.0, -512.0),
        case(1024.0, 513.0, 512.0, 1.0, -513.0),
        case(1024.0, 1.0, 512.0, 1.0, -513.0),
        case(1024.0, 2.0, 512.0, 0.5, -513.0),
        case(1024.0, 1024.0, 512.0, 1.0, -512.0)
        ::trace
    )]
    fn test_layer_translation(screen: f32, camera: f32, sprite_width: f32, speed: f32, expected: f32) {
        let window_size = WindowSize {
            height: 576.0,
            width: screen,
        };

        let camera = Vec3::new(camera, 0.0, 0.0);
        let speed = Layer { speed, ..Default::default() };
        let mut transform = Transform::default();
        move_layer_position(&window_size, &camera, sprite_width, &speed, &mut transform);
        assert_eq!(expected, transform.translation.x);
    }
}
