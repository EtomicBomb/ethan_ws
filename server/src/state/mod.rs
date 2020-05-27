// use std::sync::{Arc, Mutex};
// use crate::god_set::GodSet;
// use crate::websocket_apps::GlobalTanksGameState;
// use std::net::TcpStream;
// use web_socket::{WebSocketListener, WebSocketMessage};
// use std::thread;
//
// struct GlobalStates {
//     god_set: Arc<Mutex<GodSet>>,
//     tank: Arc<Mutex<GlobalTanksGameState>>,
// }
//
// impl GlobalStates {
//     fn new() -> Option<GlobalStates> {
//         Some(GlobalStates {
//             god_set: Arc::new(Mutex::new(GodSet::new()?)),
//             tank: Arc::new(Mutex::new(GlobalTanksGameState::new())),
//         })
//     }
//
//     fn spawn_from_new_connection(&self, location: &str, socket: TcpStream) -> Result<(), ()> {
//         let mut reader = socket.try_clone().unwrap();
//
//         let mut state: Box<dyn InternalState> =
//             match location {
//                 "/tanks" => Box::new(TanksStateInternal::new(Arc::clone(&self.tank))),
//                 "/godset" => Box::new(GodSetStateInternal::new(Arc::clone(&self.god_set))),
//                 _ => return Err(()),
//             };
//
//         thread::Builder::new().name(format!("server{}#{}", location, 3)).spawn(move || {
//             for message in WebSocketListener::new(reader) {
//                 state.do_stuff(message);
//             }
//         }).unwrap();
//
//
//         Ok(())
//     }
// }
//
//
// struct TanksStateInternal {
//     global: Arc<Mutex<GlobalTanksGameState>>,
// }
//
// impl TanksStateInternal {
//     fn new(global: Arc<Mutex<GlobalTanksGameState>>) -> TanksStateInternal {
//         TanksStateInternal { global }
//     }
// }
//
// impl InternalState for TanksStateInternal {
//     fn do_stuff(&mut self, message: WebSocketMessage) {
//         todo!()
//     }
// }
//
//
//
// struct GodSetStateInternal {
//     global: Arc<Mutex<GodSet>>,
// }
//
// impl GodSetStateInternal {
//     fn new(global: Arc<Mutex<GodSet>>) -> GodSetStateInternal {
//         GodSetStateInternal { global }
//     }
// }
//
// impl InternalState for GodSetStateInternal {
//     fn do_stuff(&mut self, message: WebSocketMessage) {
//         todo!()
//     }
// }
//
// trait InternalState: Send {
//     fn do_stuff(&mut self, message: WebSocketMessage);
// }
