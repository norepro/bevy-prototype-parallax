use bevy::prelude::*;
use bevy::window::PrimaryWindow;
///! Helpers for getting the current window size.
///! We need to know this so we can determine how much to repeat an image

/// Simple struct storing the height and width of the window.
/// Hopefully this may be integrated into bevy in future.
#[derive(Component, Default)]
pub struct WindowSize {
    pub height: f32,
    pub width: f32,
}

/// Syncs the window width to the camera
pub fn window_size(
    windows: Query<&Window, With<PrimaryWindow>>,
    mut camera_query: Query<&mut WindowSize, With<Camera>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    for mut size in camera_query.iter_mut() {
        size.width = window.resolution.width();
        size.height = window.resolution.height();
    }
}
