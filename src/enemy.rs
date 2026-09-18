use bevy::prelude::*;
use crate::map::{self, TILE_SIZE};
use crate::player::{Player, PlayerHitbox};
use crate::Progression;

#[derive(Component)]
pub struct Enemy {
    pub hp: i32,
    pub max_hp: i32,
    pub contact_damage: i32,
}

#[derive(Component)]
pub struct EnemyHitbox {
    pub half_size: Vec2,
}

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_enemy)
            .add_systems(Update, (enemy_contact_damage, cleanup_dead_enemies));
    }
}

fn spawn_enemy(mut commands: Commands) {
    let tile = IVec2::new(20, 10);
    let pos = map::map_to_world(tile);
    commands.spawn((
        Enemy {
            hp: 30,
            max_hp: 30,
            contact_damage: 8,
        },
        EnemyHitbox {
            half_size: Vec2::new(TILE_SIZE * 0.35, TILE_SIZE * 0.35),
        },
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(0.6, 0.9, 0.95),
                custom_size: Some(Vec2::splat(TILE_SIZE * 0.8)),
                ..Default::default()
            },
            transform: Transform::from_xyz(pos.x, pos.y, 8.0),
            ..Default::default()
        },
        Name::new("Enemy: Slime"),
    ));
}

fn enemy_contact_damage(
    time: Res<Time>,
    mut q_player: Query<(&mut crate::player::Player, &Transform, &PlayerHitbox)>,
    q_enemy: Query<(&Transform, &Enemy, &EnemyHitbox)>,
) {
    let (mut player, player_tf, player_hit) = match q_player.get_single_mut() {
        Ok(v) => v,
        Err(_) => return,
    };

    // Tick player's contact cooldown here to ensure consistent pacing
    player.contact_damage_cooldown.tick(time.delta());

    for (enemy_tf, enemy, enemy_hit) in q_enemy.iter() {
        if aabb_overlap(
            player_tf.translation.truncate(),
            player_hit.half_size,
            enemy_tf.translation.truncate(),
            enemy_hit.half_size,
        ) {
            if player.contact_damage_cooldown.finished() {
                player.hp = (player.hp - enemy.contact_damage).max(0);
                player
                    .contact_damage_cooldown
                    .reset();
            }
        }
    }
}

fn aabb_overlap(a_pos: Vec2, a_half: Vec2, b_pos: Vec2, b_half: Vec2) -> bool {
    (a_pos.x - b_pos.x).abs() <= (a_half.x + b_half.x)
        && (a_pos.y - b_pos.y).abs() <= (a_half.y + b_half.y)
}

fn cleanup_dead_enemies(
    mut commands: Commands,
    mut progression: ResMut<Progression>,
    q_enemy: Query<(Entity, &Enemy)>,
) {
    for (entity, enemy) in q_enemy.iter() {
        if enemy.hp <= 0 {
            // Award XP and despawn
            add_xp(&mut progression, 7);
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn add_xp(progress: &mut Progression, amount: u32) {
    progress.xp += amount;
    while progress.xp >= progress.xp_to_next {
        progress.xp -= progress.xp_to_next;
        progress.level += 1;
        // simple growth curve
        progress.xp_to_next = (progress.xp_to_next as f32 * 1.25).ceil() as u32 + 3;
    }
}

