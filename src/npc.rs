use bevy::prelude::*;
use crate::map::{self, TILE_SIZE};
use crate::player::Player;
use crate::{DialogState, Progression};
use crate::assets::GameAssets;
use crate::assets::AppStartupSet;

#[derive(Component)]
pub struct Npc {
    pub name: &'static str,
}

#[derive(Component)]
struct NpcCue; // '!' marker visibility toggled on proximity

#[derive(Component)]
struct NpcNameplate;

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
            .add_systems(Startup, spawn_npc.in_set(AppStartupSet::Spawn))
            .add_systems(Update, (handle_interact, update_npc_cue));
    }
}

fn spawn_npc(mut commands: Commands, assets: Res<GameAssets>) {
    let tile = IVec2::new(8, 6);
    let pos = map::map_to_world(tile);
    commands
        .spawn((
            Npc { name: "Guide" },
            SpriteBundle {
                texture: assets.npc_image.clone(),
                transform: Transform::from_xyz(pos.x, pos.y, 9.0),
                ..Default::default()
            },
            Name::new("NPC: Guide"),
        ))
        .with_children(|p| {
            // Nameplate
            p.spawn((
                NpcNameplate,
                Text2dBundle {
                    text: Text::from_section(
                        "Guide",
                        TextStyle {
                            font_size: 16.0,
                            color: Color::WHITE,
                            ..Default::default()
                        },
                    ),
                    transform: Transform::from_xyz(0.0, TILE_SIZE * 0.9, 20.0),
                    ..Default::default()
                },
            ));
            // '!' cue (hidden by default) - set visibility inside the bundle to avoid duplication
            p.spawn((
                NpcCue,
                Text2dBundle {
                    text: Text::from_section(
                        "!",
                        TextStyle {
                            font_size: 20.0,
                            color: Color::srgb(1.0, 0.9, 0.2),
                            ..Default::default()
                        },
                    ),
                    transform: Transform::from_xyz(0.0, TILE_SIZE * 1.4, 21.0),
                    visibility: Visibility::Hidden,
                    ..Default::default()
                },
            ));
        });
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

fn update_npc_cue(
    mut sets: ParamSet<(
        Query<&Transform, With<Player>>,
        Query<(&Transform, &Children), With<Npc>>,
    )>,
    mut q_cue: Query<&mut Visibility, With<NpcCue>>,
) {
    let player_pos = {
        let q = sets.p0();
        match q.get_single() {
            Ok(tf) => tf.translation.truncate(),
            Err(_) => return,
        }
    };
    for (npc_tf, children) in sets.p1().iter() {
        let near = npc_tf
            .translation
            .truncate()
            .distance(player_pos)
            <= TILE_SIZE * 1.2;
        for &child in children.iter() {
            if let Ok(mut vis) = q_cue.get_mut(child) {
                *vis = if near {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
            }
        }
    }
}

