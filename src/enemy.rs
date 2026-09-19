use bevy::prelude::*;
use crate::map::{self, TILE_SIZE, TileMap};
use crate::player::{Player, PlayerHitbox};
use crate::Progression;
use crate::npc::QuestState;
use crate::assets::GameAssets;

#[derive(Component)]
pub struct Enemy {
    pub hp: i32,
    pub max_hp: i32,
    pub contact_damage: i32,
    pub flash_timer: Option<Timer>,
}

#[derive(Component)]
pub struct EnemyHitbox {
    pub half_size: Vec2,
}

#[derive(Component)]
pub struct EnemyLabel;

#[derive(Component)]
pub struct EnemyQuestMarker;

#[derive(Component)]
pub struct EnemyAI {
    pub speed: f32,
    pub aggro_range: f32,
    pub attack_cooldown: Timer,
    pub knockback: Vec2,
    pub knockback_timer: Timer,
}

#[derive(Component)]
pub struct EnemySpawner {
    pub respawn_after: Timer,
    pub template: EnemyTemplate,
}

#[derive(Clone, Copy)]
pub struct EnemyTemplate {
    pub hp: i32,
    pub max_hp: i32,
    pub contact_damage: i32,
    pub speed: f32,
    pub aggro_range: f32,
}

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_enemy)
            .add_systems(
                Update,
                (
                    enemy_chase_and_attack,
                    tick_enemy_effects,
                    enemy_contact_damage.after(enemy_chase_and_attack),
                    cleanup_dead_enemies,
                    respawn_enemies,
                    sync_enemy_markers,
                ),
            );
    }
}

fn spawn_enemy(mut commands: Commands, assets: Res<GameAssets>) {
    let tile = IVec2::new(20, 10);
    let pos = map::map_to_world(tile);
    spawn_one_enemy(&mut commands, pos, &assets);
    // Spawner to bring it back after death
    commands.spawn((
        EnemySpawner {
            respawn_after: Timer::from_seconds(3.0, TimerMode::Once),
            template: EnemyTemplate {
                hp: 30,
                max_hp: 30,
                contact_damage: 8,
                speed: 70.0,
                aggro_range: TILE_SIZE * 8.0,
            },
        },
        Transform::from_xyz(pos.x, pos.y, 8.0),
        GlobalTransform::default(),
        Name::new("Spawner: Slime"),
    ));
}

fn spawn_one_enemy(commands: &mut Commands, pos: Vec2, assets: &GameAssets) {
    commands
        .spawn((
            Enemy {
                hp: 30,
                max_hp: 30,
                contact_damage: 8,
                flash_timer: None,
            },
            EnemyHitbox {
                half_size: Vec2::new(TILE_SIZE * 0.35, TILE_SIZE * 0.35),
            },
            EnemyAI {
                speed: 70.0,
                aggro_range: TILE_SIZE * 8.0,
                attack_cooldown: Timer::from_seconds(0.8, TimerMode::Once),
                knockback: Vec2::ZERO,
                knockback_timer: Timer::from_seconds(0.0, TimerMode::Once),
            },
            SpatialBundle {
                transform: Transform::from_xyz(pos.x, pos.y, 8.0),
                ..Default::default()
            },
            Name::new("Enemy: Slime"),
        ))
        .with_children(|p| {
            // Main body from texture
            p.spawn(SpriteBundle {
                texture: assets.slime_image.clone(),
                transform: Transform::from_xyz(0.0, 0.0, 0.0),
                ..Default::default()
            });
            p.spawn((
                EnemyLabel,
                Text2dBundle {
                    text: Text::from_section(
                        "Slime",
                        TextStyle {
                            font_size: 14.0,
                            color: Color::WHITE,
                            ..Default::default()
                        },
                    ),
                    transform: Transform::from_xyz(0.0, TILE_SIZE * 0.95, 20.0),
                    ..Default::default()
                },
            ));
            p.spawn((
                EnemyQuestMarker,
                Text2dBundle {
                    text: Text::from_section(
                        "●",
                        TextStyle {
                            font_size: 16.0,
                            color: Color::srgb(1.0, 0.2, 0.2),
                            ..Default::default()
                        },
                    ),
                    transform: Transform::from_xyz(0.0, TILE_SIZE * 1.25, 21.0),
                    ..Default::default()
                },
            ));
        });
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
    q_enemy: Query<(Entity, &Enemy, &Transform)>,
    mut q_spawners: Query<(&Transform, &mut EnemySpawner)>,
    mut quest: ResMut<QuestState>,
) {
    for (entity, enemy, _tf) in q_enemy.iter() {
        if enemy.hp <= 0 {
            // Award XP and despawn
            add_xp(&mut progression, 7);
            commands.entity(entity).despawn_recursive();
            // quest progress
            if quest.accepted && quest.progress < quest.target {
                quest.progress += 1;
                if quest.progress >= quest.target {
                    quest.completed = true;
                }
            }
            // Arm nearby spawner for respawn
            for (s_tf, mut spawner) in q_spawners.iter_mut() {
                if s_tf.translation.truncate().distance(_tf.translation.truncate()) < TILE_SIZE {
                    spawner.respawn_after.reset();
                }
            }
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

fn enemy_chase_and_attack(
    time: Res<Time>,
    tilemap: Res<TileMap>,
    mut sets: ParamSet<(
        Query<(&mut Transform, &EnemyHitbox, &mut EnemyAI), Without<Player>>,
        Query<&Transform, With<Player>>,
    )>,
) {
    // Extract player position first to avoid overlapping ParamSet borrows
    let player_pos = {
        let q_player = sets.p1();
        match q_player.get_single() {
            Ok(tf) => tf.translation.truncate(),
            Err(_) => return,
        }
    };
    for (mut tf, hit, mut ai) in sets.p0().iter_mut() {
        ai.attack_cooldown.tick(time.delta());
        // Knockback overrides movement
        if ai.knockback_timer.remaining_secs() > 0.0 {
            let kb_dt = ai.knockback * time.delta_seconds();
            try_move(&mut tf, hit.half_size, kb_dt, &tilemap);
            ai.knockback_timer.tick(time.delta());
            continue;
        }
        let to_player = player_pos - tf.translation.truncate();
        let dist = to_player.length();
        if dist < ai.aggro_range && dist > 1.0 {
            let dir = to_player / dist.max(1.0);
            let delta = dir * ai.speed * time.delta_seconds();
            try_move(&mut tf, hit.half_size, delta, &tilemap);
        }
    }
}

fn try_move(tf: &mut Transform, half: Vec2, delta: Vec2, tilemap: &TileMap) {
    let mut new_pos = tf.translation.truncate();
    // X
    new_pos.x += delta.x;
    if collides_with_walls(new_pos, half, tilemap) {
        new_pos.x -= delta.x;
    }
    // Y
    new_pos.y += delta.y;
    if collides_with_walls(new_pos, half, tilemap) {
        new_pos.y -= delta.y;
    }
    tf.translation.x = new_pos.x;
    tf.translation.y = new_pos.y;
}

fn collides_with_walls(pos: Vec2, half: Vec2, tilemap: &TileMap) -> bool {
    let corners = [
        Vec2::new(pos.x - half.x, pos.y - half.y),
        Vec2::new(pos.x + half.x, pos.y - half.y),
        Vec2::new(pos.x - half.x, pos.y + half.y),
        Vec2::new(pos.x + half.x, pos.y + half.y),
    ];
    for c in corners {
        let tile = map::world_to_map(c);
        if tile.x < 0
            || tile.y < 0
            || tile.x >= tilemap.width
            || tile.y >= tilemap.height
            || tilemap.walls.contains(&tile)
        {
            return true;
        }
    }
    false
}

fn tick_enemy_effects(
    time: Res<Time>,
    mut q_enemy: Query<(&mut Enemy, &mut Sprite)>,
) {
    for (mut enemy, mut sprite) in q_enemy.iter_mut() {
        if let Some(timer) = &mut enemy.flash_timer {
            timer.tick(time.delta());
            if timer.just_finished() {
                sprite.color = Color::srgb(0.8, 0.2, 0.5);
                enemy.flash_timer = None;
            } else {
                // flash tint
                sprite.color = Color::srgb(1.0, 0.4, 0.4);
            }
        }
    }
}

fn respawn_enemies(
    time: Res<Time>,
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut q_spawners: Query<(&Transform, &mut EnemySpawner)>,
    q_enemies: Query<&Transform, With<Enemy>>,
) {
    for (tf, mut spawner) in q_spawners.iter_mut() {
        spawner.respawn_after.tick(time.delta());
        if spawner.respawn_after.just_finished() {
            // Only respawn if no enemy at this spot
            let spot = tf.translation.truncate();
            let occupied = q_enemies
                .iter()
                .any(|e_tf| e_tf.translation.truncate().distance(spot) < TILE_SIZE * 0.5);
            if !occupied {
                spawn_one_enemy(&mut commands, spot, &assets);
            }
        }
    }
}

pub fn sync_enemy_markers(
    quest: Option<Res<crate::npc::QuestState>>,
    mut q: Query<&mut Visibility, With<EnemyQuestMarker>>,
) {
    let visible = if let Some(qs) = quest {
        qs.accepted && !qs.completed
    } else {
        false
    };
    for mut vis in q.iter_mut() {
        *vis = if visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

