use std::time::Duration;

use bevy::prelude::*;
use shared::{
    casting::{CasterNetId, DespawnTime, SharedCastingPlugin},
    event::{
        client::{BulletHit, SomeoneCast},
        NetEntId, ERFE,
    },
    AnyPlayer,
};

use crate::{
    cameras::notifications::Notification,
    player::{Player, PlayerName},
    states::GameState,
};

pub struct CastingNetworkPlugin;

impl Plugin for CastingNetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(SharedCastingPlugin)
            .insert_resource(HP(3))
            .add_event::<Die>()
            .add_systems(
                Update,
                (on_someone_cast, on_someone_hit, on_die)
                    .run_if(in_state(GameState::ClientConnected)),
            );
    }
}

fn on_someone_cast(
    mut someone_cast: ERFE<SomeoneCast>,
    other_players: Query<(Entity, &NetEntId, &Transform)>,
    mut commands: Commands,
    //TODO dont actually spawn a cube on cast
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for cast in someone_cast.read() {
        for (_ply_ent, ply_net_ent, ply_tfm) in &other_players {
            if &cast.event.caster_id == ply_net_ent {
                match cast.event.cast {
                    shared::event::server::Cast::Teleport(target) => {
                        info!(?target, "Someone teleported")
                    }
                    shared::event::server::Cast::Shoot(ref dat) => {
                        let cube = PbrBundle {
                            mesh: meshes.add(Mesh::from(shape::Cube { size: 0.3 })),
                            material: materials.add(Color::rgb(0.0, 0.3, 0.7).into()),
                            transform: Transform::from_translation(ply_tfm.translation),
                            ..Default::default()
                        };

                        commands.spawn((
                            cube,
                            dat.clone(),
                            cast.event.cast_id,
                            CasterNetId(cast.event.caster_id),
                            DespawnTime(Timer::new(Duration::from_secs(5), TimerMode::Once)),
                            // TODO Add a netentid for referencing this item later
                        ));
                    }
                }
            }
        }
    }
}

#[derive(Resource, Clone)]
struct HP(i32);

#[derive(Event)]
struct Die;

fn on_die(mut die: EventReader<Die>, mut me: Query<&mut Transform, With<Player>>) {
    for _death in die.read() {
        me.single_mut().translation = Vec3::new(0.0, 1.0, 0.0)
    }
}

fn on_someone_hit(
    mut someone_hit: ERFE<BulletHit>,
    all_plys: Query<(&NetEntId, &PlayerName, Has<Player>), With<AnyPlayer>>,
    mut notifs: EventWriter<Notification>,
    bullets: Query<(Entity, &NetEntId, &CasterNetId)>,
    mut temp_hp: ResMut<HP>,
    mut die: EventWriter<Die>,
    //mut commands: Commands,
) {
    for hit in someone_hit.read() {
        let mut bullet_caster_id = None;
        for (_bullet_ent, bullet_ent_id, attacker_net_id) in &bullets {
            if bullet_ent_id == &hit.event.bullet {
                bullet_caster_id = Some(attacker_net_id);
            }
        }

        // if we dont know about the bullet, return
        let bullet_caster_id = match bullet_caster_id {
            Some(s) => s.0,
            None => return warn!("Unknown bullet"),
        };

        let mut attacker_name = None;
        let mut defender_name = None;

        for (ply_id, PlayerName(name), is_us) in &all_plys {
            if ply_id == &hit.event.player {
                defender_name = Some(name);
                if is_us {
                    // TODO clientside damage!
                    temp_hp.0 -= 1;
                    if temp_hp.0 <= 0 {
                        notifs.send(Notification(format!("We died!")));
                        temp_hp.0 = 3;
                        die.send(Die);
                    } else {
                        notifs.send(Notification(format!("HP: {}", temp_hp.0)));
                    }
                }
            }

            if ply_id == &bullet_caster_id {
                attacker_name = Some(name);
            }
        }

        match (attacker_name, defender_name) {
            (Some(atk), Some(def)) => {
                info!(?atk, ?def, "Hit!");
                notifs.send(Notification(format!("{atk} hit {def}")));
            }
            (Some(atk), None) => {
                warn!(?atk, "Unknown defender");
            }
            (None, Some(def)) => {
                warn!(?def, "Unknown attacker");
            }
            (None, None) => {
                warn!("Unknown bullet");
            }
        }
    }
}