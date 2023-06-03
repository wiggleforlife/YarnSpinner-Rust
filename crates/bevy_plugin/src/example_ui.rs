use bevy::prelude::*;
use seldom_fn_plugin::FnPluginExt;

#[derive(Debug, Default)]
#[non_exhaustive]
pub struct ExampleYarnSlingerUiPlugin;

impl ExampleYarnSlingerUiPlugin {
    pub fn new() -> Self {
        Self::default()
    }
}

mod assets;
mod setup;
mod updating;

impl Plugin for ExampleYarnSlingerUiPlugin {
    fn build(&self, app: &mut App) {
        app.fn_plugin(assets::ui_assets_plugin)
            .fn_plugin(setup::ui_setup_plugin)
            .fn_plugin(updating::ui_updating_plugin);
    }
}
