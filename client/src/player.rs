use bevy::prelude::*;
//use bevy_xpbd_3d::prelude::{Collider, RigidBody};
use shared::{unit::{AttackIntention, MovementIntention}, AnyUnit};

use crate::{skills::CurrentTargetingCursor, worldgen::ChunkPos};

#[derive(Reflect, Component)]
pub struct Player {
    pub looking_at: Vec3,
    pub facing_vel: f32,
    pub velocity: Vec3,
    pub lock_movement: [Option<Vec2>; 4],
    pub current_chunk: ChunkPos,
}

#[derive(Reflect, Component, Debug)]
pub struct PlayerName(pub String);

impl Default for Player {
    fn default() -> Self {
        Self {
            // Look at camera
            looking_at: Vec3::new(10., 10., 10.),
            facing_vel: 0.,
            velocity: Vec3::ZERO,
            lock_movement: [None; 4],
            current_chunk: ChunkPos(-10, 0, -10),
        }
    }
}
#[derive(Resource)]
pub struct Animation(pub Handle<AnimationClip>);

#[derive(Component)]
pub struct PrimaryUnitControl;

pub fn spawn_player_sprite(
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,
    //mut meshes: ResMut<Assets<Mesh>>,
    //mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(Animation(asset_server.load("tadpole.gltf#Animation0")));

    commands.spawn((
        SceneBundle {
            scene: asset_server.load("tadpole.gltf#Scene0"),
            transform: Transform::from_xyz(0., 1.0, 0.)
                .with_rotation(Quat::from_rotation_y(std::f32::consts::PI))
                .with_scale(Vec3::new(4., 4., 4.)),
            ..default()
        },
        //RigidBody::Dynamic,
        //Collider::cuboid(1., 1., 1.),
        Name::new("Player"),
        MovementIntention(Vec2::ZERO),
        AttackIntention::None,
        Player::default(),
        crate::cameras::FaceCamera,
        crate::physics::Jumper {
            timer: Timer::from_seconds(1.05, TimerMode::Once),
        },
        AnyUnit,
        PrimaryUnitControl,
        SpatialListener::new(1.0),
        CurrentTargetingCursor(None),
    ));
}

//pub fn spawn_spectator_camera(
//mut commands: Commands,
//asset_server: ResMut<AssetServer>,
//) {
//}

// Once the scene is loaded, start the animation
pub fn animations(
    animations: Res<Animation>,
    mut players: Query<&mut AnimationPlayer, Added<AnimationPlayer>>,
) {
    for mut player in &mut players {
        player.play(animations.0.clone_weak()).repeat();
    }
}