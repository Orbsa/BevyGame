use bevy::prelude::*;
use bevy::reflect::Reflect;
use bevy_mod_raycast::Intersection;
use bevy_rapier3d::prelude::{ActiveEvents, Collider, RigidBody};


use crate::player::Player;
use crate::setup::MyRaycastSet;

pub fn init(app: &mut App) -> &mut App {
    app.add_system(lifetime_despawn)
        .add_system(update_all_bullets)
        .add_system(spawn_bullet)
        .add_system(tower_shooting)
        .add_system(camera_aim)
        .register_type::<Tower>()
        .add_startup_system(spawn_tower)
}

#[derive(Reflect, Component, Default)]
#[reflect(Component)]
pub struct Lifetime {
    pub timer: Timer,
}

pub fn lifetime_despawn(
    mut commands: Commands,
    mut bullets: Query<(Entity, &mut Lifetime)>,
    time: Res<Time>,
) {
    for (entity, mut lifetime) in &mut bullets {
        lifetime.timer.tick(time.delta());
        if lifetime.timer.just_finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

enum BulletAI {
    /// Bullet directly travels from point to point
    Direct,
    Wavy,
    Wavy2,
}

#[derive(Component)]
struct BulletPhysics {
    fired_from: Vec2,
    fired_target: Vec2,
    // Tiles per second
    speed: f32,
    ai: BulletAI,
    //fired_time: time_since_start,
}

fn update_all_bullets(
    mut bullets: Query<(&Lifetime, &BulletPhysics, &mut Transform)>,
    _time: Res<Time>,
) {
    for (lifetime, phys, mut transform) in bullets.iter_mut() {
        let nanos: f64 = lifetime.timer.elapsed().as_nanos() as f64;
        let secs = nanos / 1_000_000_000.0;
        let distance = (secs as f32) * phys.speed;

        let dir: Vec2 = (phys.fired_target - phys.fired_from).normalize();

        // Bullet positions are deterministic, based purely on time elapsed
        let offset: Vec2 = match phys.ai {
            BulletAI::Direct => distance * dir,
            BulletAI::Wavy => {
                let rotate_right = Vec2::new(dir.y, -dir.x);
                let wavy_offset = rotate_right * distance.sin();
                distance * dir + wavy_offset * 0.5
            }
            BulletAI::Wavy2 => {
                let rotate_right = Vec2::new(dir.x, dir.y);
                let wavy_offset = rotate_right * distance.sin();
                distance * dir + wavy_offset * 2.0
            }
        };

        // Bullets float 0.5 above the ground
        let nl: Vec2 = phys.fired_from + offset;
        *transform = Transform::from_xyz(nl.x, 0.5, nl.y);
    }
}

fn camera_aim(
    intersect: Query<&Intersection<MyRaycastSet>>,
    mut aim_target_cube: Query<&mut Transform, With<AimVectorTarget>>,
) {
    for i in &intersect {
        if let Ok(mut s) = aim_target_cube.get_single_mut() {
            match i.position() {
                Some(pos) => s.translation = *pos,
                None => s.translation = Vec3::ZERO,
            }
        }
    }
}

#[derive(Component, Reflect)]
struct AimVectorTarget;

fn spawn_bullet(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    _buttons: Res<Input<MouseButton>>,
    keyboard_input: Res<Input<KeyCode>>,
    player: Query<&Transform, With<Player>>,
    towers: Query<&Transform, With<Tower>>,
    intersect: Query<&Intersection<MyRaycastSet>>,
) {
    // Right click, red wavy, left click, blue direct
    let (color, ai) = if keyboard_input.just_pressed(KeyCode::E) {
        (Color::PINK, BulletAI::Wavy2)
    } else if keyboard_input.just_pressed(KeyCode::R) {
        (Color::RED, BulletAI::Wavy)
    } else if keyboard_input.just_pressed(KeyCode::T) {
        (Color::OLIVE, BulletAI::Direct)
    } else {
        return;
    };

    let target: &Intersection<_> = match intersect.iter().next() {
        Some(s) => s,
        None => {
            info!("No intersection with ground");
            return;
        }
    };

    info!(?target);

    let isect = match target.position() {
        Some(s) => s,
        None => {
            error!("No intersect position?");
            return;
        }
    };

    info!(?isect);

    let player_transform: &Transform = player.single();
    let _tower_transform: &Transform = towers.single();
    let spawn_transform = Transform::from_xyz(0.0, 0.5, 0.0);
    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube::new(0.4))),
            material: materials.add(color.into()),
            transform: spawn_transform,
            ..default()
        })
        .insert(Lifetime {
            timer: Timer::from_seconds(5.0, TimerMode::Once),
        })
        .insert(BulletPhysics {
            fired_target: Vec2 {
                x: isect.x,
                y: isect.z,
            },
            fired_from: Vec2 {
                x: player_transform.translation.x,
                y: player_transform.translation.z,
            },
            speed: 10.0,
            ai,
        })
        .insert(Name::new("Bullet"));
}

#[derive(Reflect, Component, Default)]
#[reflect(Component)]
pub struct Tower {
    shooting_timer: Timer,
}

fn tower_shooting(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut towers: Query<(&mut Tower, &Transform)>,
    player: Query<&Transform, With<Player>>,
    time: Res<Time>,
) {
    for (mut tower, tower_transform) in &mut towers {
        tower.shooting_timer.tick(time.delta());
        if !tower.shooting_timer.just_finished() {
            continue;
        }

        let color = Color::OLIVE;

        if let Some(player_transform) = player.iter().next() {
            let size = 0.5;
            let spawn_transform = Transform::from_xyz(0.0, 0.5, 0.0);
            commands
                .spawn((
                    PbrBundle {
                        mesh: meshes.add(Mesh::from(shape::Cube::new(size * 2.))),
                        material: materials.add(color.into()),
                        transform: spawn_transform,
                        ..default()
                    },
                    RigidBody::Dynamic,
                ))
                .insert(Collider::cuboid(size, size, size))
                .insert(Lifetime {
                    timer: Timer::from_seconds(5.0, TimerMode::Once),
                })
                .insert(BulletPhysics {
                    // make this player position
                    fired_from: Vec2 {
                        x: tower_transform.translation.x,
                        y: tower_transform.translation.z,
                    },
                    // randomize these
                    fired_target: Vec2 {
                        x: player_transform.translation.x,
                        y: player_transform.translation.z,
                    },
                    speed: 10.0,
                    ai: BulletAI::Direct,
                })
                .insert(ActiveEvents::COLLISION_EVENTS)
                .insert(Name::new("Bullet"));
        }
    }
}

fn spawn_tower(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box::new(0.5, 4., 0.5))),
            material: materials.add(Color::hex("#FF0000").unwrap().into()),
            transform: Transform::from_xyz(-4., 2., 4.),
            ..default()
        })
        .insert(Tower {
            shooting_timer: Timer::from_seconds(0.25, TimerMode::Repeating),
        })
        .insert(Name::new("Tower"));

    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube::new(0.1))),
            material: materials.add(Color::PURPLE.into()),
            transform: Transform::default(),
            ..default()
        })
        .insert(AimVectorTarget)
        .insert(Name::new("AimVector"));
}