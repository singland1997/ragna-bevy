use bevy::prelude::*;
use crate::map::{self, TILE_SIZE};
use crate::player::Player;
use crate::{DialogState, Progression};

#[derive(Component)]
pub struct Npc {
    pub name: &'static str,
}

#[derive(Resource, Default)]
pub struct QuestState {
    pub accepted: bool,
    pub target: u32,
    pub progress: u32,
    pub completed: bool,
}

pub struct NpcPlugin;

impl Plugin for NpcPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<QuestState>()
            .add_systems(Startup, spawn_npc)
            .add_systems(Update, handle_interact);
    }
}

fn spawn_npc(mut commands: Commands) {
    let tile = IVec2::new(8, 6);
    let pos = map::map_to_world(tile);
    commands.spawn((
        Npc { name: "Town Guide" },
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(1.0, 0.85, 0.2),
                custom_size: Some(Vec2::splat(TILE_SIZE * 0.8)),
                ..Default::default()
            },
            transform: Transform::from_xyz(pos.x, pos.y, 9.0),
            ..Default::default()
        },
        Name::new("NPC: Town Guide"),
    ));
}

fn handle_interact(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut dialog: ResMut<DialogState>,
    mut quest: ResMut<QuestState>,
    mut progression: ResMut<Progression>,
    q_player: Query<&Transform, With<Player>>,
    q_npc: Query<(&Transform, &Npc)>,
) {
    let player_tf = match q_player.get_single() {
        Ok(v) => v,
        Err(_) => return,
    };
    for (npc_tf, npc) in q_npc.iter() {
        let dist = player_tf
            .translation
            .truncate()
            .distance(npc_tf.translation.truncate());
        let near = dist <= TILE_SIZE * 1.2;
        if near && (keyboard.just_pressed(KeyCode::KeyE) || keyboard.just_pressed(KeyCode::Space)) {
            dialog.open = !dialog.open;
            if dialog.open {
                // Dialogue depending on quest state
                if !quest.accepted && !quest.completed {
                    quest.accepted = true;
                    quest.target = 5;
                    quest.progress = 0;
                    dialog.text = format!(
                        "{}: Welcome to Asteria Meadow!\nI have a task for you: defeat {} slimes.\nReturn when done for a small reward.",
                        npc.name, quest.target
                    );
                } else if quest.accepted && !quest.completed {
                    dialog.text = format!(
                        "{}: Progress: {}/{}. Keep going!",
                        npc.name, quest.progress, quest.target
                    );
                } else if quest.completed {
                    dialog.text = format!("{}: Well done! Take this bonus XP.", npc.name);
                    // simple one-time reward
                    progression.xp += 10;
                    quest.completed = false; // allow repeating lightly
                }
            } else {
                dialog.text.clear();
            }
        }
    }
}

