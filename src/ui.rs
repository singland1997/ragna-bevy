use bevy::prelude::*;
use crate::{Progression, DialogState, DeathState};
use crate::player::Player;
use crate::skills::SkillState;
use crate::npc::QuestState;

const HUD_BAR_WIDTH: f32 = 220.0;
const HUD_BAR_HEIGHT: f32 = 16.0;

#[derive(Component)]
struct HpBarFg;

#[derive(Component)]
struct XpBarFg;

#[derive(Component)]
struct LevelText;

#[derive(Component)]
struct CooldownText;

#[derive(Component)]
struct DialogRoot;

#[derive(Component)]
struct QuestText;

#[derive(Component)]
struct ControlsHintRoot;

#[derive(Resource)]
pub struct ControlsHint {
    pub timer: Timer,
}

impl Default for ControlsHint {
    fn default() -> Self {
        Self { timer: Timer::from_seconds(6.0, TimerMode::Once) }
    }
}

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ControlsHint>()
            .add_systems(Startup, setup_ui)
            .add_systems(Update, (update_hp_bar, update_xp_bar, update_texts, sync_dialog_visibility, tick_controls_hint, sync_death_overlay));
    }
}

fn setup_ui(mut commands: Commands) {
    // HUD root (top-left)
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    left: Val::Px(16.0),
                    top: Val::Px(16.0),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(8.0),
                    ..Default::default()
                },
                background_color: BackgroundColor(Color::NONE),
                ..Default::default()
            },
            Name::new("HUD Root"),
        ))
        .with_children(|parent| {
            // HP label + bar
            parent.spawn(TextBundle::from_section(
                "HP",
                TextStyle { font_size: 14.0, color: Color::WHITE, ..Default::default() },
            ));
            parent.spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Px(HUD_BAR_WIDTH),
                        height: Val::Px(HUD_BAR_HEIGHT),
                        ..Default::default()
                    },
                    background_color: BackgroundColor(Color::srgb(0.2, 0.0, 0.02)),
                    ..Default::default()
                },
                Name::new("HP Bar Background"),
            ))
            .with_children(|bar| {
                bar.spawn((
                    NodeBundle {
                        style: Style {
                            width: Val::Px(HUD_BAR_WIDTH),
                            height: Val::Px(HUD_BAR_HEIGHT),
                            ..Default::default()
                        },
                        background_color: BackgroundColor(Color::srgb(0.8, 0.15, 0.2)),
                        ..Default::default()
                    },
                    HpBarFg,
                    Name::new("HP Bar Fill"),
                ));
            });

            // Level + XP text
            parent.spawn((
                TextBundle::from_section(
                    "Lvl 1  XP 0/10  Quest: —",
                    TextStyle {
                        font_size: 16.0,
                        color: Color::WHITE,
                        ..Default::default()
                    },
                ),
                LevelText,
                Name::new("Level/XP Text"),
            ));

            // XP label + bar
            parent.spawn(TextBundle::from_section(
                "XP",
                TextStyle { font_size: 14.0, color: Color::WHITE, ..Default::default() },
            ));
            parent
                .spawn((
                    NodeBundle {
                        style: Style {
                            width: Val::Px(HUD_BAR_WIDTH),
                            height: Val::Px(HUD_BAR_HEIGHT),
                            ..Default::default()
                        },
                        background_color: BackgroundColor(Color::srgb(0.10, 0.10, 0.22)),
                        ..Default::default()
                    },
                    Name::new("XP Bar Background"),
                ))
                .with_children(|bar| {
                    bar.spawn((
                        NodeBundle {
                            style: Style {
                                width: Val::Px(0.0),
                                height: Val::Px(HUD_BAR_HEIGHT),
                                ..Default::default()
                            },
                            background_color: BackgroundColor(Color::srgb(0.2, 0.4, 0.9)),
                            ..Default::default()
                        },
                        XpBarFg,
                        Name::new("XP Bar Fill"),
                    ));
                });

            // Cooldown text
            parent.spawn((
                TextBundle::from_section(
                    "Q: Ready",
                    TextStyle {
                        font_size: 16.0,
                        color: Color::WHITE,
                        ..Default::default()
                    },
                ),
                CooldownText,
                Name::new("Cooldown Text"),
            ));
        });

    // Dialog overlay (bottom)
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    left: Val::Px(24.0),
                    right: Val::Px(24.0),
                    bottom: Val::Px(24.0),
                    padding: UiRect::all(Val::Px(12.0)),
                    ..Default::default()
                },
                visibility: Visibility::Hidden,
                background_color: BackgroundColor(Color::srgb(0.05, 0.05, 0.08)),
                ..Default::default()
            },
            DialogRoot,
            Name::new("Dialog Box"),
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "Dialog",
                TextStyle {
                    font_size: 18.0,
                    color: Color::WHITE,
                    ..Default::default()
                },
            ));
        });

    // Controls hint overlay (top-center)
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    left: Val::Percent(25.0),
                    right: Val::Percent(25.0),
                    top: Val::Px(20.0),
                    padding: UiRect::all(Val::Px(8.0)),
                    justify_content: JustifyContent::Center,
                    ..Default::default()
                },
                background_color: BackgroundColor(Color::srgb(0.02, 0.02, 0.04)),
                ..Default::default()
            },
            ControlsHintRoot,
            Name::new("Controls Hint"),
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "WASD/Arrows: Move   E/Space: Talk   Q: Skill   J: Melee",
                TextStyle {
                    font_size: 16.0,
                    color: Color::WHITE,
                    ..Default::default()
                },
            ));
        });
}

fn update_hp_bar(
    q_player: Query<&Player>,
    mut q_bar: Query<&mut Style, With<HpBarFg>>,
) {
    let player = match q_player.get_single() {
        Ok(v) => v,
        Err(_) => return,
    };
    let frac = (player.hp as f32 / player.max_hp as f32).clamp(0.0, 1.0);
    if let Ok(mut style) = q_bar.get_single_mut() {
        style.width = Val::Px(HUD_BAR_WIDTH * frac);
    }
}

fn update_xp_bar(
    progression: Res<Progression>,
    mut q_bar: Query<&mut Style, With<XpBarFg>>,
) {
    if let Ok(mut style) = q_bar.get_single_mut() {
        let frac = (progression.xp as f32 / progression.xp_to_next as f32).clamp(0.0, 1.0);
        style.width = Val::Px(HUD_BAR_WIDTH * frac);
    }
}

fn update_texts(
    progression: Res<Progression>,
    skill: Res<SkillState>,
    quest: Option<Res<QuestState>>,
    mut texts: ParamSet<(
        Query<&mut Text, With<LevelText>>,
        Query<&mut Text, With<CooldownText>>,
    )>,
) {
    if let Ok(mut txt) = texts.p0().get_single_mut() {
        let quest_str = if let Some(q) = quest {
            if q.accepted && !q.completed {
                format!("Quest: {}/{}", q.progress, q.target)
            } else if q.completed {
                "Quest: Complete!".to_string()
            } else {
                "Quest: —".to_string()
            }
        } else {
            "Quest: —".to_string()
        };
        txt.sections[0].value =
            format!("Lvl {}  XP {}/{}  {}", progression.level, progression.xp, progression.xp_to_next, quest_str);
    }
    if let Ok(mut txt) = texts.p1().get_single_mut() {
        if skill.cooldown.finished() {
            txt.sections[0].value = "Q: Ready".to_string();
        } else {
            let remaining = skill.cooldown.remaining().as_secs_f32();
            txt.sections[0].value = format!("Q: {:.1}s", remaining.max(0.0));
        }
    }
}

fn sync_dialog_visibility(
    dialog: Res<DialogState>,
    mut q_dialog: Query<(&mut Visibility, &Children), With<DialogRoot>>,
    mut q_text: Query<&mut Text>,
) {
    if !dialog.is_changed() {
        return;
    }
    if let Ok((mut vis, children)) = q_dialog.get_single_mut() {
        *vis = if dialog.open {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        // Update dialog text
        for &child in children.iter() {
            if let Ok(mut text) = q_text.get_mut(child) {
                if dialog.open {
                    text.sections[0].value = format!("{}\n(Press E or Space to close)", dialog.text);
                }
            }
        }
    }
}

fn tick_controls_hint(
    time: Res<Time>,
    mut hint: ResMut<ControlsHint>,
    mut q: Query<&mut BackgroundColor, With<ControlsHintRoot>>,
) {
    hint.timer.tick(time.delta());
    let alpha = if hint.timer.finished() {
        0.0
    } else {
        // fade out over time
        (1.0 - hint.timer.elapsed_secs() / hint.timer.duration().as_secs_f32()).clamp(0.0, 1.0)
    };
    if let Ok(mut bg) = q.get_single_mut() {
        let c = bg.0.to_srgba();
        *bg = BackgroundColor(Color::srgba(c.red, c.green, c.blue, alpha));
    }
}

#[derive(Component)]
struct DeathOverlayRoot;

fn sync_death_overlay(
    death: Res<DeathState>,
    mut commands: Commands,
    q_overlay: Query<Entity, With<DeathOverlayRoot>>,
) {
    let should_show = death.dead;
    let exists = q_overlay.get_single().ok();
    match (should_show, exists) {
        (true, None) => {
            // Spawn overlay
            commands
                .spawn((
                    NodeBundle {
                        style: Style {
                            position_type: PositionType::Absolute,
                            left: Val::Px(0.0),
                            right: Val::Px(0.0),
                            top: Val::Px(0.0),
                            bottom: Val::Px(0.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..Default::default()
                        },
                        background_color: BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
                        ..Default::default()
                    },
                    DeathOverlayRoot,
                    Name::new("Death Overlay"),
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "You fell... Respawning...",
                        TextStyle {
                            font_size: 28.0,
                            color: Color::WHITE,
                            ..Default::default()
                        },
                    ));
                });
        }
        (false, Some(e)) => {
            commands.entity(e).despawn_recursive();
        }
        _ => {}
    }
}

