use bevy::prelude::*;
use crate::{Progression, DialogState};
use crate::player::Player;
use crate::skills::SkillState;

const HUD_BAR_WIDTH: f32 = 220.0;
const HUD_BAR_HEIGHT: f32 = 16.0;

#[derive(Component)]
struct HpBarFg;

#[derive(Component)]
struct LevelText;

#[derive(Component)]
struct CooldownText;

#[derive(Component)]
struct DialogRoot;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_ui)
            .add_systems(Update, (update_hp_bar, update_texts, sync_dialog_visibility));
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
            // HP bar
            parent
                .spawn((
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
                    "Lvl 1  XP 0/10",
                    TextStyle {
                        font_size: 16.0,
                        color: Color::WHITE,
                        ..Default::default()
                    },
                ),
                LevelText,
                Name::new("Level/XP Text"),
            ));

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

fn update_texts(
    progression: Res<Progression>,
    skill: Res<SkillState>,
    mut q_lvl: Query<&mut Text, With<LevelText>>,
    mut q_cd: Query<&mut Text, With<CooldownText>>,
) {
    if let Ok(mut txt) = q_lvl.get_single_mut() {
        txt.sections[0].value = format!(
            "Lvl {}  XP {}/{}",
            progression.level, progression.xp, progression.xp_to_next
        );
    }
    if let Ok(mut txt) = q_cd.get_single_mut() {
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

