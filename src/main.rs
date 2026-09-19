mod map;
mod player;
mod npc;
mod enemy;
mod combat;
mod skills;
mod ui;
mod assets;

use bevy::prelude::*;
use combat::CombatPlugin;
use enemy::EnemyPlugin;
use map::MapPlugin;
use npc::NpcPlugin;
use player::PlayerPlugin;
use skills::SkillsPlugin;
use ui::UiPlugin;
use assets::AssetsPlugin;

pub const GAME_TITLE: &str = "Asteria Meadow";

#[derive(Resource, Default)]
pub struct Progression {
    pub level: u32,
    pub xp: u32,
    pub xp_to_next: u32,
}

#[derive(Resource, Default)]
pub struct DialogState {
    pub open: bool,
    pub text: String,
}

#[derive(Resource, Default)]
pub struct DeathState {
    pub dead: bool,
    pub timer: Timer,
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.05, 0.07, 0.10)))
        .insert_resource(Progression {
            level: 1,
            xp: 0,
            xp_to_next: 10,
        })
        .insert_resource(DialogState::default())
        .insert_resource(DeathState {
            dead: false,
            timer: Timer::from_seconds(2.0, TimerMode::Once),
        })
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: GAME_TITLE.to_string(),
                ..Default::default()
            }),
            ..Default::default()
        }))
        .add_systems(Startup, setup_camera)
        .add_plugins((
            AssetsPlugin,
            MapPlugin,
            PlayerPlugin,
            NpcPlugin,
            EnemyPlugin,
            CombatPlugin,
            SkillsPlugin,
            UiPlugin,
        ))
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
