use std::{sync::{Mutex, Arc}, ops::DerefMut, str::from_utf8};

use bevy::{app::ScheduleRunnerSettings, prelude::*, utils::Duration, log::LogPlugin};
use message_io::{node, network::{Transport, NetEvent, Endpoint}};
use shared::{GameNetEvent, event::PlayerConnect};

fn main() {
    info!("Main Start");
    let mut app = App::new();

    app
        .insert_resource(ScheduleRunnerSettings::run_loop(Duration::from_secs_f64(1.0 / 120.0)))
        .insert_resource(ServerResources::default())
        .add_event::<shared::event::PlayerConnect>()
        .add_plugins(MinimalPlugins)
        .add_plugin(LogPlugin::default())
        .add_startup_system(start_server)
        .add_system(on_player_connect)
        .add_system(tick_server);

    app.run();
}

#[derive(Resource, Default, Debug, Clone)]
struct ServerResources {
    event_list: Arc<Mutex<Vec<(Endpoint, GameNetEvent)>>>,
}

fn start_server(
    event_list_res: Res<ServerResources>,
) {
    warn!("Start Server");

    let (handler, listener) = node::split::<()>();
    let res_copy = event_list_res.clone();

    std::thread::spawn(move || {
        handler.network().listen(Transport::Udp, "0.0.0.0:3042").unwrap();

        listener.for_each(move |event| match event.network() {
            NetEvent::Connected(_, _) => unreachable!(),
            NetEvent::Accepted(_endpoint, _listener) => println!("Client connected"),
            NetEvent::Message(endpoint, data) => {
                //let s = from_utf8(data);
                //info!(?s);
                let event = serde_json::from_slice(data).unwrap();
                res_copy.event_list.lock().unwrap().push((endpoint, event));
            },
            NetEvent::Disconnected(_endpoint) => println!("Client disconnected"),
        });
    });

}

fn tick_server(
    event_list_res: Res<ServerResources>,
    mut ev_player_connect: EventWriter<PlayerConnect>,
) {
    let events_to_process = std::mem::take(event_list_res.event_list.lock().unwrap().deref_mut());
    for event in events_to_process {
        let (_endpoint, e) = event;
        match e {
            GameNetEvent::Noop => warn!("Got noop event"),
            GameNetEvent::PlayerConnect(p) => ev_player_connect.send(p),
        }
    }
}

fn on_player_connect(
    mut ev_player_connect: EventReader<PlayerConnect>,
) {
    for e in &mut ev_player_connect {
        info!("Got a player connection event {e:?}");
    }
}
