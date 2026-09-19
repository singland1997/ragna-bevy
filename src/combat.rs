use bevy::prelude::*;
use crate::enemy::{Enemy, EnemyHitbox};
use crate::skills::Projectile;
use crate::player::{Player, PlayerHitbox, Facing};
use crate::map::TILE_SIZE;

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                move_projectiles,
                projectile_hits_enemy,
                cleanup_projectiles,
                melee_attack,
                melee_hits_enemy,
                tick_damage_numbers,
            ),
        );
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
                enemy.flash_timer = Some(Timer::from_seconds(0.12, TimerMode::Once));
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

#[derive(Component)]
struct MeleeHitbox {
    dir: Vec2,
    life: Timer,
    damage: i32,
    half_size: Vec2,
}

fn melee_attack(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut commands: Commands,
    mut q_player: Query<(&mut Player, &Transform)>,
) {
    let (mut player, tf) = match q_player.get_single_mut() {
        Ok(v) => v,
        Err(_) => return,
    };
    if (keyboard.just_pressed(KeyCode::KeyJ) || mouse.just_pressed(MouseButton::Left))
        && player.melee_cooldown.finished()
    {
        let dir = match player.facing {
            Facing::Up => Vec2::Y,
            Facing::Down => -Vec2::Y,
            Facing::Left => -Vec2::X,
            Facing::Right => Vec2::X,
        };
        let start = tf.translation.truncate() + dir * (TILE_SIZE * 0.7);
        // Visible flash
        commands.spawn((
            MeleeHitbox {
                dir,
                life: Timer::from_seconds(0.08, TimerMode::Once),
                damage: 10,
                half_size: Vec2::new(TILE_SIZE * 0.5, TILE_SIZE * 0.5),
            },
            SpriteBundle {
                sprite: Sprite {
                    color: Color::srgb(1.0, 0.95, 0.6),
                    custom_size: Some(Vec2::splat(TILE_SIZE * 0.7)),
                    ..Default::default()
                },
                transform: Transform::from_xyz(start.x, start.y, 12.0),
                ..Default::default()
            },
            Name::new("Melee Swing"),
        ));
        player
            .melee_cooldown
            .set_duration(bevy::utils::Duration::from_secs_f32(0.35));
        player.melee_cooldown.reset();
    }
}

fn melee_hits_enemy(
    time: Res<Time>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut q_enemy: Query<(&Transform, &EnemyHitbox, &mut Enemy, &mut crate::enemy::EnemyAI)>,
    mut q_melee: Query<(Entity, &Transform, &mut MeleeHitbox)>,
) {
    for (entity, tf, mut melee) in q_melee.iter_mut() {
        melee.life.tick(time.delta());
        let mut any_hit = false;
        for (enemy_tf, enemy_hit, mut enemy, mut ai) in q_enemy.iter_mut() {
            if aabb_overlap(
                tf.translation.truncate(),
                melee.half_size,
                enemy_tf.translation.truncate(),
                enemy_hit.half_size,
            ) {
                enemy.hp -= melee.damage;
                enemy.flash_timer = Some(Timer::from_seconds(0.1, TimerMode::Once));
                // Knockback
                ai.knockback = melee.dir * 220.0;
                ai.knockback_timer = Timer::from_seconds(0.08, TimerMode::Once);
                ai.knockback_timer.reset();
                // Floating damage number
                spawn_damage_number(&mut commands, &asset_server, enemy_tf.translation.truncate(), melee.damage);
                any_hit = true;
            }
        }
        if melee.life.finished() || any_hit {
            commands.entity(entity).despawn_recursive();
        }
    }
}

#[derive(Component)]
struct DamageNumber {
    timer: Timer,
    velocity: Vec2,
}

fn spawn_damage_number(
    commands: &mut Commands,
    asset_server: &AssetServer,
    pos: Vec2,
    amount: i32,
) {
    let text_style = TextStyle {
        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
        font_size: 18.0,
        color: Color::srgb(1.0, 0.9, 0.3),
    };
    commands.spawn((
        DamageNumber {
            timer: Timer::from_seconds(0.6, TimerMode::Once),
            velocity: Vec2::new(0.0, 40.0),
        },
        Text2dBundle {
            text: Text::from_section(format!("{}", amount), text_style),
            transform: Transform::from_xyz(pos.x, pos.y + TILE_SIZE * 0.6, 30.0),
            ..Default::default()
        },
        Name::new("Damage Number"),
    ));
}

fn tick_damage_numbers(
    time: Res<Time>,
    mut commands: Commands,
    mut q: Query<(Entity, &mut DamageNumber, &mut Transform, &mut Text)>,
) {
    for (e, mut dn, mut tf, mut text) in q.iter_mut() {
        dn.timer.tick(time.delta());
        tf.translation.y += dn.velocity.y * time.delta_seconds();
        // Fade out
        let remaining = dn.timer.remaining_secs().max(0.0);
        let frac = remaining / 0.6;
        for section in &mut text.sections {
            let s = section.style.color.to_srgba();
            section.style.color = Color::srgba(s.red, s.green, s.blue, frac as f32);
        }
        if dn.timer.just_finished() {
            commands.entity(e).despawn_recursive();
        }
    }
}

