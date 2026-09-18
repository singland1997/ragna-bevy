use bevy::prelude::*;
use crate::enemy::{Enemy, EnemyHitbox};
use crate::skills::Projectile;

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (move_projectiles, projectile_hits_enemy, cleanup_projectiles));
    }
}

fn move_projectiles(
    time: Res<Time>,
    mut q_projectiles: Query<(&mut Transform, &Projectile)>,
) {
    let dt = time.delta_seconds();
    for (mut tf, proj) in q_projectiles.iter_mut() {
        let delta = proj.dir * proj.speed * dt;
        tf.translation.x += delta.x;
        tf.translation.y += delta.y;
    }
}

fn projectile_hits_enemy(
    mut commands: Commands,
    mut q_enemy: Query<(&Transform, &mut Enemy, &EnemyHitbox)>,
    q_projectiles: Query<(Entity, &Transform, &Projectile)>,
) {
    for (proj_entity, proj_tf, proj) in q_projectiles.iter() {
        let mut hit = false;
        for (enemy_tf, mut enemy, enemy_hit) in q_enemy.iter_mut() {
            if aabb_overlap(
                proj_tf.translation.truncate(),
                proj.half_size,
                enemy_tf.translation.truncate(),
                enemy_hit.half_size,
            ) {
                enemy.hp -= proj.damage;
                hit = true;
                break;
            }
        }
        if hit {
            commands.entity(proj_entity).despawn_recursive();
        }
    }
}

fn cleanup_projectiles(
    time: Res<Time>,
    mut commands: Commands,
    mut q_projectiles: Query<(Entity, &mut crate::skills::Projectile)>,
) {
    for (entity, mut proj) in q_projectiles.iter_mut() {
        proj.life.tick(time.delta());
        if proj.life.finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn aabb_overlap(a_pos: Vec2, a_half: Vec2, b_pos: Vec2, b_half: Vec2) -> bool {
    (a_pos.x - b_pos.x).abs() <= (a_half.x + b_half.x)
        && (a_pos.y - b_pos.y).abs() <= (a_half.y + b_half.y)
}

