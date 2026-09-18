use bevy::prelude::*;
use crate::player::{Facing, Player};
use crate::map::TILE_SIZE;

#[derive(Resource)]
pub struct SkillState {
    pub base_cooldown: f32,
    pub cooldown: Timer,
}

impl Default for SkillState {
    fn default() -> Self {
        Self {
            base_cooldown: 1.5,
            cooldown: Timer::from_seconds(0.0, TimerMode::Once),
        }
    }
}

#[derive(Component)]
pub struct Projectile {
    pub dir: Vec2,
    pub speed: f32,
    pub life: Timer,
    pub damage: i32,
    pub half_size: Vec2,
}

pub struct SkillsPlugin;

impl Plugin for SkillsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SkillState>()
            .add_systems(Update, (tick_cooldown, cast_projectile));
    }
}

fn tick_cooldown(time: Res<Time>, mut skill: ResMut<SkillState>) {
    skill.cooldown.tick(time.delta());
}

fn cast_projectile(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut skill: ResMut<SkillState>,
    q_player: Query<(&Transform, &crate::player::Player)>,
    mut commands: Commands,
) {
    let (player_tf, player) = match q_player.get_single() {
        Ok(v) => v,
        Err(_) => return,
    };
    if keyboard.just_pressed(KeyCode::KeyQ) && skill.cooldown.finished() {
        let dir = match player.facing {
            Facing::Up => Vec2::Y,
            Facing::Down => -Vec2::Y,
            Facing::Left => -Vec2::X,
            Facing::Right => Vec2::X,
        };
        let start = player_tf.translation.truncate() + dir * (TILE_SIZE * 0.7);
        commands.spawn((
            Projectile {
                dir,
                speed: 360.0,
                life: Timer::from_seconds(0.8, TimerMode::Once),
                damage: 12,
                half_size: Vec2::splat(TILE_SIZE * 0.25),
            },
            SpriteBundle {
                sprite: Sprite {
                    color: Color::srgb(1.0, 0.3, 0.3),
                    custom_size: Some(Vec2::splat(TILE_SIZE * 0.5)),
                    ..Default::default()
                },
                transform: Transform::from_xyz(start.x, start.y, 11.0),
                ..Default::default()
            },
            Name::new("Projectile: Bolt"),
        ));
        // Reset cooldown
        skill
            .cooldown
            .set_duration(bevy::utils::Duration::from_secs_f32(skill.base_cooldown));
        skill.cooldown.reset();
    } else {
        // Keep ticking so we don't freeze timers if no input
        let _ = time.delta_seconds();
    }
}

