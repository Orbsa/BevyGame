use std::{
    collections::HashMap,
    sync::{atomic::AtomicI16, Arc},
    time::Duration,
};

use bevy::{log::LogPlugin, prelude::*};
use bevy_time::common_conditions::on_timer;
use message_io::network::Endpoint;
use rand::Rng;
use shared::{
    event::{
        client::{PlayerDisconnected, WorldData},
        NetEntId, ERFE,
    },
    netlib::{send_event_to_server, EventToClient, EventToServer, ServerResources},
    Config, ConfigPlugin,
};

const HEARTBEAT_MILLIS: u64 = 200;

#[derive(States, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
enum ServerState {
    #[default]
    Starting,
    Running,
}

#[derive(Resource, Default)]
struct HeartbeatList {
    heartbeats: HashMap<NetEntId, Arc<AtomicI16>>,
}

#[derive(Resource, Default)]
struct EndpointToNetId {
    map: HashMap<Endpoint, NetEntId>,
}

#[derive(Component)]
struct ConnectedPlayer;

#[derive(Component)]
struct PlayerEndpoint(Endpoint);

#[derive(Event)]
struct PlayerDisconnect {
    ent: NetEntId,
}

fn main() {
    info!("Main Start");
    let mut app = App::new();

    shared::event::server::register_events(&mut app);
    app.add_plugins(ConfigPlugin)
        .insert_resource(EndpointToNetId::default())
        .insert_resource(HeartbeatList::default())
        .add_event::<PlayerDisconnect>()
        .add_plugins(MinimalPlugins)
        .add_plugins(LogPlugin::default())
        .add_state::<ServerState>()
        .add_systems(
            Startup,
            (
                shared::netlib::setup_server::<EventToServer>,
                //on_player_connect,
                //tick_net_server,
                //send_shooting_to_all_players,
                //send_animations_to_all_players,
                //send_movement_to_all_players,
                |mut state: ResMut<NextState<ServerState>>| state.set(ServerState::Running),
            ),
        )
        .add_systems(
            Update,
            (
                shared::event::server::drain_events,
                on_player_connect,
                on_player_disconnect,
            )
                .run_if(in_state(ServerState::Running)),
        )
        .add_systems(
            Update,
            check_heartbeats.run_if(on_timer(Duration::from_millis(200))),
        );

    app.run();
}

fn on_player_connect(
    mut new_players: ERFE<shared::event::server::ConnectRequest>,
    mut heartbeat_mapping: ResMut<HeartbeatList>,
    sr: Res<ServerResources<EventToServer>>,
    _config: Res<Config>,
    mut commands: Commands,
) {
    for player in new_players.read() {
        info!(?player);
        let name = player
            .event
            .name
            .clone()
            .unwrap_or_else(|| format!("Player #{}", rand::thread_rng().gen_range(1..10000)));

        let ent_id = NetEntId(rand::random());
        let event = EventToClient::WorldData(WorldData {
            your_name: name,
            your_id: ent_id,
        });
        send_event_to_server(&sr.handler, player.endpoint, &event);

        commands.spawn((ConnectedPlayer, ent_id, PlayerEndpoint(player.endpoint)));
        heartbeat_mapping
            .heartbeats
            .insert(ent_id, Arc::new(AtomicI16::new(0)));
    }
}

fn check_heartbeats(
    heartbeat_mapping: ResMut<HeartbeatList>,
    mut on_disconnect: EventWriter<PlayerDisconnect>,
) {
    for (ent_id, beats_missed) in &heartbeat_mapping.heartbeats {
        let beats = beats_missed.fetch_add(1, std::sync::atomic::Ordering::Acquire);
        if beats >= (5000 / HEARTBEAT_MILLIS) as i16 {
            warn!("Missed {beats} beats, disconnecting {ent_id:?}");
            on_disconnect.send(PlayerDisconnect { ent: *ent_id });
        }
    }
}

fn on_player_disconnect(
    mut pd: EventReader<PlayerDisconnect>,
    clients: Query<(Entity, &PlayerEndpoint, &NetEntId), With<ConnectedPlayer>>,
    //mut commands: Commands,
    mut heartbeat_mapping: ResMut<HeartbeatList>,
    sr: Res<ServerResources<EventToServer>>,
) {
    for player in pd.read() {
        heartbeat_mapping.heartbeats.remove(&player.ent);

        let event = EventToClient::PlayerDisconnected(PlayerDisconnected { id: player.ent });
        for (_c_ent, c_net_client, _c_net_ent) in &clients {
            send_event_to_server(&sr.handler, c_net_client.0, &event);
        }

        //for (c_ent, c_net_client, _c_net_ent) in &clients {
        //event_list_res
        //.handler
        //.network()
        //.send(c_net_client.endpoint, events_as_str.as_bytes());

        //if _c_net_ent == ent_id {
        //commands.entity(c_ent).despawn_recursive();
        //}
        //}
    }
}

//for (_endpoint, e) in events_to_process {
//match e {
//EventToServer::Noop => todo!(),
//EventToServer::Connect { name } => {
//info!("A player has connected with name '{name}'");
//let id = NetEntId(thread_rng().gen());
//let ev = PlayerInfo {
//name,
//id,
//};

//ev_player_connect.send(EventFromEndpoint::new(_endpoint, ev));

//entity_mapping.map.insert(_endpoint, id);
//// give them 5 seconds to connect properly
//let start_heartbeat = 5000 / (HEARTBEAT_MILLIS as i64);
//heartbeat_mapping.heartbeats.insert(id, Arc::new(AtomicI16::new(-start_heartbeat as _)));
//},
//EventToServer::UpdatePos(new_pos) => {
//match entity_mapping.map.get(&_endpoint) {
//Some(id) => {
//ev_player_movement.send(UpdatePos {
//id: *id,
//transform: new_pos,
//});
//}
//None => {} // error!("Failed to match endpoint {_endpoint:?}to id"),
//}
//},
//EventToServer::ShootBullet(phys) => {
//match entity_mapping.map.get(&_endpoint) {
//Some(id) => {
//debug!("Player {id:?} is shooting");
//ev_player_shooting.send(ShootBullet {
//id: *id,
//phys,
//});
//}
//None => error!("Failed to match endpoint {_endpoint:?}to id"),
//}
//}
//EventToServer::BeginAnimation(animation) => {
//match entity_mapping.map.get(&_endpoint) {
//Some(id) => {
//info!("Player {id:?} is animating");
//ev_player_animating.send(Animation {
//id: *id,
//animation,
//});
//}
//None => error!("Failed to match endpoint {_endpoint:?}to id"),
//}
//}
//EventToServer::Heartbeat => {
//match entity_mapping.map.get(&_endpoint) {
//Some(id) => {
//heartbeat_mapping.heartbeats
//.get(id)
//.unwrap()
//.store(0, std::sync::atomic::Ordering::Release);
//}
//None => error!("Failed to match endpoint {_endpoint:?}to id"),
//}
//}

//_ => todo!(),
//}
//}
//}

//fn send_movement_to_all_players(
//mut ev_player_movement: EventReader<UpdatePos>,
//event_list_res: Res<ServerResources<EventToServer>>,
//players: Query<&GameNetClient>,
//) {
//let events: Vec<_> = ev_player_movement
//.into_iter()
//.map(|x| EventToClient::UpdatePos(x.clone()))
//.collect();

//for client in &players {
//for event in &events {
//let events_as_str = serde_json::to_string(&event).unwrap();
//event_list_res.handler
//.network()
//.send(client.endpoint, events_as_str.as_bytes());
//}
//}
//}

//fn send_shooting_to_all_players(
//mut ev_shoot: EventReader<ShootBullet>,
//event_list_res: Res<ServerResources<EventToServer>>,
//players: Query<&GameNetClient>,
//) {
//let events: Vec<_> = ev_shoot
//.iter()
//.map(|x| EventToClient::ShootBullet(x.clone()))
//.collect();

//for client in &players {
//for event in &events {
//let events_as_str = serde_json::to_string(&event).unwrap();
//event_list_res.handler
//.network()
//.send(client.endpoint, events_as_str.as_bytes());
//}
//}
//}

//fn send_animations_to_all_players(
//mut ev_animate: EventReader<Animation>,
//event_list_res: Res<ServerResources<EventToServer>>,
//players: Query<(Entity, &GameNetClient, &NetEntId)>,
//mut commands: Commands,
//) {

//for event in ev_animate.iter() {
//commands
//.spawn(event.clone());

//let events_as_str = serde_json::to_string(&EventToClient::Animation(event.clone())).unwrap();
//for (ent, client, net_id) in &players {
//event_list_res.handler
//.network()
//.send(client.endpoint, events_as_str.as_bytes());

//}
//}
//}

//#[derive(Component)]
//struct GameNetClient {
//name: String,
//endpoint: Endpoint,
//}

//fn on_player_connect(
//mut ev_player_connect: EventReader<EventFromEndpoint<PlayerInfo>>,
//other_players: Query<(&GameNetClient, &NetEntId)>,
//mut commands: Commands,
//event_list_res: Res<ServerResources<EventToServer>>,
//) {
//for e in &mut ev_player_connect {
//info!("Got a player connection event {e:?}");
//let new_client = GameNetClient {
//endpoint: e.endpoint,
//name: e.event.name.clone(),
//};

//// First, notify all existing players about the new player
//// Also collect all their names to use later
//let connect_event = EventToClient::PlayerConnect(e.event.clone());
//let mut names = vec![];
//for (player, ent_id) in &other_players {
//let data = serde_json::to_string(&connect_event).unwrap();
//event_list_res.handler
//.network()
//.send(player.endpoint, data.as_bytes());

//names.push(PlayerInfo {
//name: player.name.clone(),
//id: *ent_id,
//});
//}

//// Next, tell the new player who they are
//let connect_event = EventToClient::YouAre(PlayerInfo {
//name: e.event.name.clone(),
//id: e.event.id,
//});
//let handler = event_list_res.handler.clone();
//let data = serde_json::to_string(&connect_event).unwrap();
//handler
//.network()
//.send(e.endpoint, data.as_bytes());

//// Next, tell the new player about the existing players
//let connect_event = EventToClient::PlayerList(names);
//let data = serde_json::to_string(&connect_event).unwrap();
//event_list_res.handler
//.network()
//.send(e.endpoint, data.as_bytes());

//// Finally, add our client to the ECS
//commands
//.spawn_empty()
//.insert(new_client)
//.insert(e.event.id);
//}
//}
