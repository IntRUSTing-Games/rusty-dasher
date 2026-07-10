use bevy::prelude::*;

/// Audio + small UI icons (hearts). Gameplay entities stay mesh-based.
#[derive(Resource)]
pub struct GameAssets {
    pub sfx_collect: Handle<AudioSource>,
    pub sfx_dash: Handle<AudioSource>,
    pub sfx_hit: Handle<AudioSource>,
    pub sfx_powerup: Handle<AudioSource>,
    pub sfx_menu: Handle<AudioSource>,
    pub sfx_gameover: Handle<AudioSource>,
    pub sfx_levelup: Handle<AudioSource>,
    pub tex_heart: Handle<Image>,
}

impl FromWorld for GameAssets {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        Self {
            sfx_collect: asset_server.load("sounds/collect.ogg"),
            sfx_dash: asset_server.load("sounds/dash.ogg"),
            sfx_hit: asset_server.load("sounds/hit.ogg"),
            sfx_powerup: asset_server.load("sounds/powerup.ogg"),
            sfx_menu: asset_server.load("sounds/menu.ogg"),
            sfx_gameover: asset_server.load("sounds/gameover.ogg"),
            sfx_levelup: asset_server.load("sounds/levelup.ogg"),
            tex_heart: asset_server.load("sprites/heart.png"),
        }
    }
}

pub fn play_sfx(commands: &mut Commands, handle: Handle<AudioSource>) {
    commands.spawn((AudioPlayer::new(handle), PlaybackSettings::DESPAWN));
}
