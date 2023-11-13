use std::f32::consts::PI;

use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
};
use shared::Config;

use crate::{physics::Jumper, player::Player};

use super::FreeCamState;

#[derive(Reflect, Component)]
pub struct CameraFollow {
    pub distance: f32,
    pub min_distance: f32,
    pub max_distance: f32,
    pub dragging: bool,
    pub yaw_radians: f32,
    pub pitch_radians: f32,
    pub old_yaw: f32,
}
impl Default for CameraFollow {
    fn default() -> Self {
        Self {
            distance: 10.,
            min_distance: 2.,
            max_distance: 200.,
            dragging: false,
            yaw_radians: 0.,
            pitch_radians: PI * 1.0 / 4.0,
            old_yaw: 0.,
        }
    }
}

pub fn q_e_rotate_cam(
    keyboard_input: Res<Input<KeyCode>>,
    mut camera_query: Query<&mut CameraFollow>,
    time: Res<Time>,
    config: Res<Config>,
) {
    let mut rotation = 0.0;
    if keyboard_input.pressed(KeyCode::Q) {
        rotation += 1.0;
    }
    if keyboard_input.pressed(KeyCode::E) {
        rotation -= 1.0;
    }
    if rotation != 0.0 {
        camera_query.single_mut().yaw_radians += config.qe_sens * rotation * time.delta_seconds();
    }
}

pub fn wow_camera_system(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut mouse_events: EventReader<MouseMotion>,
    mouse_input: Res<Input<MouseButton>>,
    mut camera_query: Query<(&mut Transform, &mut CameraFollow), With<Camera3d>>,
    player_query: Query<&Transform, (With<Player>, Without<CameraFollow>)>,
    _keyboard_input: Res<Input<KeyCode>>,
    camera_type: Res<State<FreeCamState>>,
    config: Res<Config>,
) {
    let player_transform = match player_query.get_single() {
        Ok(s) => s,
        Err(_) => return,
    };

    for (mut camera_transform, mut camera_follow) in camera_query.iter_mut() {
        for event in mouse_wheel_events.read() {
            camera_follow.distance -= event.y;
            camera_follow.distance = camera_follow
                .distance
                .clamp(camera_follow.min_distance, camera_follow.max_distance);
        }

        if mouse_input.pressed(MouseButton::Right)
            || *camera_type == FreeCamState::ThirdPersonLocked
        {
            for event in mouse_events.read() {
                let sens = config.sens;
                camera_follow.yaw_radians -= event.delta.x * sens;
                camera_follow.pitch_radians -= event.delta.y * sens;
                camera_follow.pitch_radians =
                    camera_follow.pitch_radians.clamp(0.05 * PI, 0.95 * PI);
            }
        }

        let camera_location = Quat::from_rotation_y(camera_follow.yaw_radians)
            * Quat::from_rotation_z(camera_follow.pitch_radians)
            * Vec3::Y
            * camera_follow.distance
            + player_transform.translation;

        let new_transform = Transform::from_translation(camera_location)
            .looking_at(player_transform.translation + 0.5 * Vec3::Y, Vec3::Y);

        camera_transform.translation = new_transform.translation;
        camera_transform.rotation = new_transform.rotation;
    }
}

pub const PLAYER_SPEED: f32 = 5.;
pub fn player_movement(
    _commands: Commands,
    mut player_query: Query<(&mut Transform, Entity, &mut Jumper, &mut Player)>,
    camera_query: Query<&CameraFollow>,
    keyboard_input: Res<Input<KeyCode>>,
    config: Res<Config>,
    time: Res<Time>,
) {
    for (mut transform, _player_ent, mut jumper, _player) in player_query.iter_mut() {
        let mut move_vector = Vec2::ZERO;
        if config.pressing_keybind(
            |x| keyboard_input.pressed(x),
            shared::GameAction::MoveForward,
        ) {
            move_vector += Vec2::new(1.0, 0.0);
        }
        if config.pressing_keybind(
            |x| keyboard_input.pressed(x),
            shared::GameAction::MoveBackward,
        ) {
            move_vector += Vec2::new(-1.0, 0.0);
        }
        if config.pressing_keybind(
            |x| keyboard_input.pressed(x),
            shared::GameAction::StrafeLeft,
        ) {
            move_vector += Vec2::new(0.0, -1.0);
        }
        if config.pressing_keybind(
            |x| keyboard_input.pressed(x),
            shared::GameAction::StrafeRight,
        ) {
            move_vector += Vec2::new(0.0, 1.0);
        }

        jumper.timer.tick(time.delta());
        if keyboard_input.pressed(KeyCode::Space) {
            if jumper.timer.finished() {
                // TODO jump
                jumper.timer.reset();
            }
        }

        if move_vector.length_squared() > 0.0 {
            let camera = camera_query.single();
            let rotation = Vec2::from_angle(-camera.yaw_radians);
            let movem = move_vector.normalize().rotate(rotation);

            transform.translation +=
                Vec3::new(movem.x, 0.0, movem.y) * PLAYER_SPEED * time.delta_seconds();
        }
    }
}