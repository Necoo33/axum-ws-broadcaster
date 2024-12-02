#[cfg(feature = "typed")]
pub mod typed {
    use serde::Serialize;
    use futures_util::{Sink, Stream, SinkExt, stream::{SplitSink, SplitStream, StreamExt}};
    use std::{fmt::Display, sync::Arc};
    use tokio::sync::RwLock;
    use axum_typed_websockets::{Message, WebSocket};

    #[derive(Debug)]
    pub struct Broadcaster<T, S> {
        pub rooms: Vec<Room<T, S>>
    }

    #[derive(Debug)]
    pub struct Room<T, S> {
        pub id: String,
        pub connections: Vec<Connection<T, S>>
    }

    #[derive(Debug)]
    pub struct Connection<T, S> {
        pub id: String,
        pub receiver: SplitSink<WebSocket<T, S>, Message<T>>
    }

    impl<T: Display, S: Display> Connection<T, S> where SplitSink<WebSocket<T, S>, Message<T>>: Sink<Message<T>> + Unpin {
        pub fn create(id: String, receiver: SplitSink<WebSocket<T, S>, Message<T>>) -> Self {
            Self {
                id, 
                receiver
            }
        }
    }

    impl<T: Display + Serialize, S: Display + Serialize> Room<T, S> where SplitSink<WebSocket<T, S>, Message<T>>: Sink<Message<T>> + Unpin {
        pub fn add_connection(&mut self, id: String, receiver: SplitSink<WebSocket<T, S>, Message<T>>) {
            let check_is_connection_exist = self.connections.iter().any(|room| room.id == id);

            match check_is_connection_exist {
                true => (),
                false => {
                    let connection = Connection {
                        id,
                        receiver
                    };

                    self.connections.push(connection);
                }
            }
        }

        pub fn remove_connection(&mut self, id: String) {
            self.connections.retain(|connection| { 
                if connection.id == id { 
                    false 
                } else { 
                    true 
                }
            });
        }

        pub fn check_connection(&mut self, id: &String) -> Option<&Connection<T, S>> {
            let connection = self.connections.iter().find(|room| room.id == *id);

            match connection {
                Some(connection) => Some(connection),
                None => None
            }
        }

        pub async fn broadcast(&mut self, message: T) where T: Clone { 
            for connection in &mut self.connections { 
                let msg = Message::Item(message.clone());
                let receiver = &mut connection.receiver; 
                
                let _ = receiver.send(msg).await;
            }
        }

        pub async fn broadcast_if<F>(&mut self, message: T, condition: F) where F: Fn(&Connection<T, S>) -> bool, T: Clone { 
            for connection in &mut self.connections { 
                if condition(connection) { 
                    let msg = Message::Item(message.clone()); 
                    let receiver = &mut connection.receiver; 
                    let _ = receiver.send(msg).await;
                } 
            } 
        }

        pub async fn broadcast_if_not<F>(&mut self, message: T, condition: F) where F: Fn(&Connection<T, S>) -> bool, T: Clone { 
            for connection in &mut self.connections { 
                if !condition(connection) { 
                    let msg = Message::Item(message.clone()); 
                    let receiver = &mut connection.receiver; 
                    let _ = receiver.send(msg).await;
                } 
            } 
        }
    }

    impl<T: Display + Serialize, S: Display + Serialize> Broadcaster<T, S> {
        pub fn new() -> Arc<RwLock<Self>> {
            Arc::new(RwLock::new(Self::default()))
        }

        pub fn configure(socket: WebSocket<T, S>) -> (SplitSink<WebSocket<T, S>, Message<T>>, SplitStream<WebSocket<T, S>>) where WebSocket<T, S>: Sink<Message<T>> + Stream + Sized {
            socket.split()
        }

        pub async fn handle(broadcaster: &Arc<RwLock<Self>>, room_id: String, conn_id: String, receiver: SplitSink<WebSocket<T, S>, Message<T>>) -> Arc<RwLock<Self>> {
            let mut broadcaster_write = broadcaster.write().await;

            broadcaster_write.handle_room(room_id).add_connection(conn_id, receiver);

            Arc::clone(&broadcaster)
        }

        pub fn handle_room(&mut self, id: String) -> &mut Room<T, S> {
            if let Some(index) = self.rooms.iter().position(|room| room.id == id) {
                return &mut self.rooms[index];
            }
        
            self.rooms.push(Room {
                id,
                connections: vec![],
            });
        
            self.rooms.last_mut().unwrap()
        }

        pub fn room(&mut self, id: String) -> &mut Room<T, S> {
            return self.rooms.iter_mut().find(|room| room.id == *id).unwrap();
        }

        pub fn check_room(&mut self, id: &String) -> Option<&mut Room<T, S>> {
            match self.rooms.iter_mut().find(|room| room.id == *id) {
                Some(room) => Some(room),
                None => None
            }
        }

        pub fn check(&self, id: &String) -> bool {
            return self.rooms.iter().any(|room| room.id == *id);
        }

        /// it removes a room with given id.
        pub fn remove_room(&mut self, id: String) {
            self.rooms.retain(|room| { 
                if room.id == id { 
                    false 
                } else { 
                    true 
                }
            });
        }

        /// it removes all empty rooms.
        pub fn remove_empty_rooms(&mut self) {
            self.rooms.retain(|room| { 
                if room.connections.is_empty() { 
                    false 
                } else { 
                    true 
                }
            });
        }

        pub fn remove_connection(&mut self, id: String) -> Option<SplitSink<WebSocket<T, S>, Message<T>>> {
            for room in &mut self.rooms {
                if let Some(pos) = room.connections.iter().position(|connection| connection.id == id) {
                    let connection = room.connections.remove(pos);
                    return Some(connection.receiver);
                }
            }
            None
        }
    }

    impl<T, S> Default for Broadcaster<T, S> { 
        fn default() -> Self { 
            Self { 
                rooms: vec![], 
            } 
        }
    }
}

pub mod normal {
    use axum::extract::ws::{Message, WebSocket};
    use std::sync::Arc;
    use tokio::sync::RwLock;
    use futures_util::{sink::SinkExt, stream::{SplitSink, SplitStream, StreamExt}};

    #[derive(Debug)]
    pub struct Broadcaster {
        pub rooms: Vec<Room>
    }

    #[derive(Debug)]
    pub struct Room {
        pub id: String,
        pub connections: Vec<Connection>
    }

    #[derive(Debug)]
    pub struct Connection {
        pub id: String,
        pub receiver: SplitSink<WebSocket, Message>
    }

    impl Connection {
        pub fn create(id: String, receiver: SplitSink<WebSocket, Message>) -> Self {
            Self {
                id, 
                receiver
            }
        }
    }

    impl Room {
        pub fn add_connection(&mut self, id: String, receiver: SplitSink<WebSocket, Message>) {
            let check_is_connection_exist = self.connections.iter().any(|room| room.id == id);

            match check_is_connection_exist {
                true => (),
                false => {
                    let connection = Connection {
                        id,
                        receiver
                    };

                    self.connections.push(connection);
                }
            }
        }

        pub fn remove_connection(&mut self, id: String) {
            self.connections.retain(|connection| { 
                if connection.id == id { 
                    false 
                } else { 
                    true 
                }
            });
        }

        pub fn check_connection(&mut self, id: &String) -> Option<&Connection> {
            let connection = self.connections.iter().find(|room| room.id == *id);

            match connection {
                Some(connection) => Some(connection),
                None => None
            }
        }

        pub async fn broadcast(&mut self, message: String) { 
            for connection in &mut self.connections { 
                let msg = Message::Text(message.clone());
                let receiver = &mut connection.receiver; 
                
                let _ = receiver.send(msg).await;
            }
        }

        pub async fn broadcast_if<F>(&mut self, message: String, condition: F) where F: Fn(&Connection) -> bool, { 
            for connection in &mut self.connections { 
                if condition(connection) { 
                    let msg = Message::Text(message.clone()); 
                    let receiver = &mut connection.receiver; 
                    let _ = receiver.send(msg).await;
                } 
            } 
        }

        pub async fn broadcast_if_not<F>(&mut self, message: String, condition: F) where F: Fn(&Connection) -> bool { 
            for connection in &mut self.connections { 
                if !condition(connection) { 
                    let msg = Message::Text(message.clone()); 
                    let receiver = &mut connection.receiver; 
                    let _ = receiver.send(msg).await;
                } 
            } 
        }
    }

    impl Broadcaster {
        pub fn new() -> Arc<RwLock<Self>> {
            Arc::new(RwLock::new(Self::default()))
        }

        pub fn configure(socket: WebSocket) -> (SplitSink<WebSocket, Message>, SplitStream<WebSocket>) {
            socket.split()
        }

        pub async fn handle(broadcaster: &Arc<RwLock<Self>>, room_id: String, conn_id: String, receiver: SplitSink<WebSocket, Message>) -> Arc<RwLock<Self>> {
            let mut broadcaster_write = broadcaster.write().await;

            broadcaster_write.handle_room(room_id).add_connection(conn_id, receiver);

            Arc::clone(&broadcaster)
        }

        pub fn handle_room(&mut self, id: String) -> &mut Room {
            if let Some(index) = self.rooms.iter().position(|room| room.id == id) {
                return &mut self.rooms[index];
            }
        
            self.rooms.push(Room {
                id,
                connections: vec![],
            });
        
            self.rooms.last_mut().unwrap()
        }

        pub fn room(&mut self, id: String) -> &mut Room {
            return self.rooms.iter_mut().find(|room| room.id == *id).unwrap();
        }

        pub fn check_room(&mut self, id: &String) -> Option<&mut Room> {
            match self.rooms.iter_mut().find(|room| room.id == *id) {
                Some(room) => Some(room),
                None => None
            }
        }

        pub fn check(&self, id: &String) -> bool {
            return self.rooms.iter().any(|room| room.id == *id);
        }

        /// it removes a room with given id.
        pub fn remove_room(&mut self, id: String) {
            self.rooms.retain(|room| { 
                if room.id == id { 
                    false 
                } else { 
                    true 
                }
            });
        }

        /// it removes all empty rooms.
        pub fn remove_empty_rooms(&mut self) {
            self.rooms.retain(|room| { 
                if room.connections.is_empty() { 
                    false 
                } else { 
                    true 
                }
            });
        }

        pub fn remove_connection(&mut self, id: String) -> Option<SplitSink<WebSocket, Message>> {
            for room in &mut self.rooms {
                if let Some(pos) = room.connections.iter().position(|connection| connection.id == id) {
                    let connection = room.connections.remove(pos);
                    return Some(connection.receiver);
                }
            }
            None
        }
    }

    impl Default for Broadcaster { 
        fn default() -> Self { 
            Self { 
                rooms: vec![], 
            } 
        }
    }
}