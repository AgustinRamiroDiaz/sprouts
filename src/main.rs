use bevy::window::WindowMode;
use bevy::{asset::AssetMetaCheck, prelude::*};
fn main() {
    let mut app = App::new();
    app.add_plugins((DefaultPlugins
        .set(AssetPlugin {
            meta_check: AssetMetaCheck::Never, // https://github.com/bevyengine/bevy/issues/10157#issuecomment-2217168402
            ..default()
        })
        .set(ImagePlugin::default_nearest())
        .set(WindowPlugin {
            primary_window: Some(Window {
                // Borderless looks distorted when running in the web
                #[cfg(not(target_arch = "wasm32"))]
                mode: WindowMode::BorderlessFullscreen(MonitorSelection::Current),
                fit_canvas_to_parent: true,
                prevent_default_event_handling: false,
                resizable: true,
                ..default()
            }),
            ..default()
        }),));
}
