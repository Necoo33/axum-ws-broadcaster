#[cfg(feature = "typed")]
pub mod typed {
    use serde::Serialize;
    use futures_util::{Sink, Stream, SinkExt, stream::{SplitSink, SplitStream, StreamExt}};
    use std::{fmt::Display, sync::Arc};
    use tokio::sync::RwLock;
    use axum_typed_websockets::{Message, WebSocket};
    use axum_7_9::extract::ws::CloseFrame;

    /// main broadcaster for typed api.
    #[derive(Debug)]
    pub struct Broadcaster<T, S> {
        pub rooms: Vec<Room<T, S>>
    }

    /// room implementation.
    #[derive(Debug)]
    pub struct Room<T, S> {
        pub id: String,
        pub connections: Vec<Connection<T, S>>
    }

    /// type for each individual connection.
    #[derive(Debug)]
    pub struct Connection<T, S> {
        pub id: String,
        pub receiver: SplitSink<WebSocket<T, S>, Message<T>>
    }

    impl<T: Display, S: Display> Connection<T, S> where SplitSink<WebSocket<T, S>, Message<T>>: Sink<Message<T>> + Unpin {
        /// create a connection:
        pub fn create(id: String, receiver: SplitSink<WebSocket<T, S>, Message<T>>) -> Self {
            Self {
                id, 
                receiver
            }
        }
    }

    impl<T: Display + Serialize, S: Display + Serialize> Room<T, S> where SplitSink<WebSocket<T, S>, Message<T>>: Sink<Message<T>> + Unpin {
        /// check if a connection with given id exist and if it's not, add a connection to a room with that ip:
        pub fn add_connection(&mut self, id: &String, receiver: SplitSink<WebSocket<T, S>, Message<T>>) {
            let check_is_connection_exist = self.connections.iter().any(|room| room.id == *id);

            match check_is_connection_exist {
                true => (),
                false => {
                    let connection = Connection {
                        id: id.clone(),
                        receiver
                    };

                    self.connections.push(connection);
                }
            }
        }

        /// remove a connection from room:
        pub fn remove_connection(&mut self, id: String) {
            self.connections.retain(|connection| { 
                if connection.id == id { 
                    false 
                } else { 
                    true 
                }
            });
        }

        /// check if a connection exist and return if it's.
        pub fn check_connection(&mut self, id: &String) -> Option<&Connection<T, S>> {
            let connection = self.connections.iter().find(|room| room.id == *id);

            match connection {
                Some(connection) => Some(connection),
                None => None
            }
        }

        /// broadcast the message directly:
        pub async fn broadcast(&mut self, message: &T) where T: Clone { 
            for connection in &mut self.connections { 
                let msg = Message::Item(message.clone());
                let receiver = &mut connection.receiver; 
                
                let _ = receiver.send(msg).await;
            }
        }

        /// broadcast the message if the given condition in it's closure is true.
        pub async fn broadcast_if<F>(&mut self, message: &T, condition: F) where F: Fn(&Connection<T, S>) -> bool, T: Clone { 
            for connection in &mut self.connections { 
                if condition(connection) { 
                    let msg = Message::Item(message.clone()); 
                    let receiver = &mut connection.receiver; 
                    let _ = receiver.send(msg).await;
                } 
            } 
        }

        /// broadcast the message if the given condition in it's closure is false.
        pub async fn broadcast_if_not<F>(&mut self, message: &T, condition: F) where F: Fn(&Connection<T, S>) -> bool, T: Clone { 
            for connection in &mut self.connections { 
                if !condition(connection) { 
                    let msg = Message::Item(message.clone()); 
                    let receiver = &mut connection.receiver; 
                    let _ = receiver.send(msg).await;
                } 
            } 
        }

        /// broadcast the message directly:
        pub async fn ping(&mut self, message: &Vec<u8>) where T: Clone {
            for connection in &mut self.connections { 
                let msg = Message::Ping(message.clone());
                let receiver = &mut connection.receiver; 
                        
                let _ = receiver.send(msg).await;
            }
        }

        /// broadcast the message if the given condition in it's closure is true.
        pub async fn ping_if<F>(&mut self, message: &Vec<u8>, condition: F) where F: Fn(&Connection<T, S>) -> bool, T: Clone { 
            for connection in &mut self.connections { 
                if condition(connection) { 
                    let msg = Message::Ping(message.clone()); 
                    let receiver = &mut connection.receiver; 
                    let _ = receiver.send(msg).await;
                } 
            } 
        }
        
        /// broadcast the message if the given condition in it's closure is false.
        pub async fn ping_if_not<F>(&mut self, message: &Vec<u8>, condition: F) where F: Fn(&Connection<T, S>) -> bool, T: Clone { 
            for connection in &mut self.connections { 
                if !condition(connection) { 
                    let msg = Message::Ping(message.clone()); 
                    let receiver = &mut connection.receiver; 
                    let _ = receiver.send(msg).await;
                } 
            } 
        }

        /// broadcast the pong message directly:
        pub async fn pong(&mut self, message: &Vec<u8>) where T: Clone {
            for connection in &mut self.connections { 
                let msg = Message::Pong(message.clone());
                let receiver = &mut connection.receiver; 
                                
                let _ = receiver.send(msg).await;
            }
        }
        
        /// broadcast the pong message if the given condition in it's closure is true.
        pub async fn pong_if<F>(&mut self, message: &Vec<u8>, condition: F) where F: Fn(&Connection<T, S>) -> bool, T: Clone { 
            for connection in &mut self.connections { 
                if condition(connection) { 
                    let msg = Message::Pong(message.clone()); 
                    let receiver = &mut connection.receiver; 
                    let _ = receiver.send(msg).await;
                } 
            } 
        }
                
        /// broadcast the pong message if the given condition in it's closure is false.
        pub async fn pong_if_not<F>(&mut self, message: &Vec<u8>, condition: F) where F: Fn(&Connection<T, S>) -> bool, T: Clone { 
            for connection in &mut self.connections { 
                if !condition(connection) { 
                    let msg = Message::Pong(message.clone()); 
                    let receiver = &mut connection.receiver; 
                    let _ = receiver.send(msg).await;
                } 
            } 
        }

        /// it's most convenient way to close a single connection but keeping room open.
        pub async fn close_conn(&mut self, close_frame: Option<CloseFrame<'static>>, id: &String) where T: Clone {
            self.connections.retain_mut(|connection| {
                if connection.id == *id {
                    let msg = Message::Close(close_frame.clone());
                    let receiver = &mut connection.receiver; 
                                                    
                    let _ = async {
                        let _ = receiver.send(msg).await;
                    };
                    
                    false
                } else {
                    true
                }
            });
        }

        /// Close all connections and remove it from it's room but not close it.
        pub async fn close(&mut self, close_frame: Option<CloseFrame<'static>>) where T: Clone { 
            self.connections.retain_mut(|connection| {
                let msg = Message::Close(close_frame.clone());
                let receiver = &mut connection.receiver; 
                                                        
                let _ = async {
                    let _ = receiver.send(msg).await;
                };
                        
                false
            });
        }
        
        /// close each connection and remove them from room if the given condition in it's closure is true.
        pub async fn close_if<F>(&mut self, close_frame: Option<CloseFrame<'static>>, condition: F) where F: Fn(&Connection<T, S>) -> bool, T: Clone { 
            self.connections.retain_mut(|connection| {
                if condition(&connection) {
                    let msg = Message::Close(close_frame.clone());
                    let receiver = &mut connection.receiver; 
                                                            
                    let _ = async {
                        let _ =  receiver.send(msg).await;
                    };
                            
                    false
                } else {
                    true
                }
            });
        }
        
        /// close each connection and remove them from room if the given condition in it's closure is false.
        pub async fn close_if_not<F>(&mut self, close_frame: Option<CloseFrame<'static>>, condition: F) where F: Fn(&Connection<T, S>) -> bool, T: Clone { 
            self.connections.retain_mut(|connection| {
                if !condition(&connection) {
                    let msg = Message::Close(close_frame.clone());
                    let receiver = &mut connection.receiver; 
                                                                    
                    let _ = async {
                        let _ =  receiver.send(msg).await;
                    };
                                    
                    false
                } else {
                    true
                }
            });
        }
    }

    impl<T: Display + Serialize, S: Display + Serialize> Broadcaster<T, S> {
        /// create new broadcaster:
        pub fn new() -> Arc<RwLock<Self>> {
            Arc::new(RwLock::new(Self::default()))
        }

        /// get receiver and stream, similar to ".handle()" method of actix-ws.
        pub fn configure(socket: WebSocket<T, S>) -> (SplitSink<WebSocket<T, S>, Message<T>>, SplitStream<WebSocket<T, S>>) where WebSocket<T, S>: Sink<Message<T>> + Stream + Sized {
            socket.split()
        }

        /// handle the all thing. If you use that api, there is no need to any other configuration for grouping and identifying connections:
        pub async fn handle(broadcaster: &Arc<RwLock<Self>>, room_id: &String, conn_id: &String, receiver: SplitSink<WebSocket<T, S>, Message<T>>) -> Arc<RwLock<Self>> {
            let mut broadcaster_write = broadcaster.write().await;

            broadcaster_write.handle_room(room_id).add_connection(conn_id, receiver);

            Arc::clone(&broadcaster)
        }

        /// check if a room with given id exist and if it's not create one.
        pub fn handle_room(&mut self, id: &String) -> &mut Room<T, S> {
            if let Some(index) = self.rooms.iter().position(|room| room.id == *id) {
                return &mut self.rooms[index];
            }
        
            self.rooms.push(Room {
                id: id.clone(),
                connections: vec![],
            });
        
            self.rooms.last_mut().unwrap()
        }

        /// Get the Room with given id. If there is a risk of unextistance of the room, use ".check_room()" instead.
        pub fn room(&mut self, id: &String) -> &mut Room<T, S> {
            return self.rooms.iter_mut().find(|room| room.id == *id).unwrap();
        }

        /// check if a room with given id exist and wrap it in an option.
        pub fn check_room(&mut self, id: &String) -> Option<&mut Room<T, S>> {
            match self.rooms.iter_mut().find(|room| room.id == *id) {
                Some(room) => Some(room),
                None => None
            }
        }

        /// only check if a room is exist and return a bool.
        pub fn check(&self, id: &String) -> bool {
            return self.rooms.iter().any(|room| room.id == *id);
        }

        /// it removes a room with given id and closes all the connections inside of it.
        pub async fn remove_room(&mut self, id: &String) where T: Clone {
            self.rooms.retain_mut(|room| { 
                if room.id == *id {
                    let _ = async {
                        let _ = room.close(None).await;
                    };
                    
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

        /// Removes the connection from Room. Warning: Because the async closures are not stable yet, we cannot close the connection in that function, you have to make cleanup on your cadebase. For that, check the examples & Documentation.
        pub fn remove_connection(&mut self, id: &String) -> Option<SplitSink<WebSocket<T, S>, Message<T>>> {
            for room in &mut self.rooms {
                if let Some(pos) = room.connections.iter().position(|connection| connection.id == *id) {
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
    use axum_8_1::{body::Bytes, extract::ws::{CloseFrame, Message, Utf8Bytes, WebSocket}};
    use std::sync::Arc;
    use tokio::sync::RwLock;
    use futures_util::{sink::SinkExt, stream::{SplitSink, SplitStream, StreamExt}};

    /// main broadcaster for normal api.
    #[derive(Debug)]
    pub struct Broadcaster {
        pub rooms: Vec<Room>
    }

    /// room implementation.
    #[derive(Debug)]
    pub struct Room {
        pub id: String,
        pub connections: Vec<Connection>
    }

    /// type for each individual connection.
    #[derive(Debug)]
    pub struct Connection {
        pub id: String,
        pub receiver: SplitSink<WebSocket, Message>
    }

    impl Connection {
        /// create a connection:
        pub fn create(id: String, receiver: SplitSink<WebSocket, Message>) -> Self {
            Self {
                id, 
                receiver
            }
        }
    }

    impl Room {
        /// check if a connection with given id exist and if it's not, add a connection to a room with that ip:
        pub fn add_connection(&mut self, id: &String, receiver: SplitSink<WebSocket, Message>) {
            let check_is_connection_exist = self.connections.iter().any(|room| room.id == *id);

            match check_is_connection_exist {
                true => (),
                false => {
                    let connection = Connection {
                        id: id.clone(),
                        receiver
                    };

                    self.connections.push(connection);
                }
            }
        }

        /// remove a connection from room with given id.
        pub fn remove_connection(&mut self, id: String) {
            self.connections.retain(|connection| { 
                if connection.id == id { 
                    false 
                } else { 
                    true 
                }
            });
        }

        /// check if a connection exist and return if it's in an option.
        pub fn check_connection(&mut self, id: &String) -> Option<&Connection> {
            let connection = self.connections.iter().find(|room| room.id == *id);

            match connection {
                Some(connection) => Some(connection),
                None => None
            }
        }

        /// Broadcast the message directly.
        pub async fn broadcast(&mut self, message: &Utf8Bytes) { 
            for connection in &mut self.connections { 
                let msg = Message::Text(message.clone());
                let receiver = &mut connection.receiver; 
                
                let _ = receiver.send(msg).await;
            }
        }

        /// broadcast the message if the given condition in it's closure is true.
        pub async fn broadcast_if<F>(&mut self, message: &Utf8Bytes, condition: F) where F: Fn(&Connection) -> bool, { 
            for connection in &mut self.connections { 
                if condition(connection) { 
                    let msg = Message::Text(message.clone()); 
                    let receiver = &mut connection.receiver; 
                    let _ = receiver.send(msg).await;
                } 
            } 
        }

        /// broadcast the message if the given condition in it's closure is false.
        pub async fn broadcast_if_not<F>(&mut self, message: &Utf8Bytes, condition: F) where F: Fn(&Connection) -> bool { 
            for connection in &mut self.connections { 
                if !condition(connection) { 
                    let msg = Message::Text(message.clone()); 
                    let receiver = &mut connection.receiver; 
                    let _ = receiver.send(msg).await;
                } 
            } 
        }

        /// Broadcast the ping message directly.
        pub async fn ping(&mut self, bytes: &Bytes) { 
            for connection in &mut self.connections { 
                let msg = Message::Ping(bytes.clone());
                let receiver = &mut connection.receiver; 
                        
                let _ = receiver.send(msg).await;
            }
        }
        
        /// broadcast the ping message if the given condition in it's closure is true.
        pub async fn ping_if<F>(&mut self, bytes: &Bytes, condition: F) where F: Fn(&Connection) -> bool, { 
            for connection in &mut self.connections { 
                if condition(connection) { 
                    let msg = Message::Ping(bytes.clone());
                    let receiver = &mut connection.receiver; 
                    let _ = receiver.send(msg).await;
                } 
            } 
        }
        
        /// broadcast the ping message if the given condition in it's closure is false.
        pub async fn ping_if_not<F>(&mut self, bytes: &Bytes, condition: F) where F: Fn(&Connection) -> bool { 
            for connection in &mut self.connections { 
                if !condition(connection) { 
                    let msg = Message::Ping(bytes.clone()); 
                    let receiver = &mut connection.receiver; 
                    let _ = receiver.send(msg).await;
                } 
            } 
        }

        /// Broadcast the pong message directly.
        pub async fn pong(&mut self, bytes: &Bytes) { 
            for connection in &mut self.connections { 
                let msg = Message::Pong(bytes.clone());
                let receiver = &mut connection.receiver; 
                                
                let _ = receiver.send(msg).await;
            }
        }
                
        /// broadcast the pong message if the given condition in it's closure is true.
        pub async fn pong_if<F>(&mut self, bytes: &Bytes, condition: F) where F: Fn(&Connection) -> bool, { 
            for connection in &mut self.connections { 
                if condition(connection) { 
                    let msg = Message::Pong(bytes.clone());
                    let receiver = &mut connection.receiver; 
                    let _ = receiver.send(msg).await;
                } 
            } 
        }
                
        /// broadcast the pong message if the given condition in it's closure is false.
        pub async fn pong_if_not<F>(&mut self, bytes: &Bytes, condition: F) where F: Fn(&Connection) -> bool { 
            for connection in &mut self.connections { 
                if !condition(connection) { 
                    let msg = Message::Pong(bytes.clone()); 
                    let receiver = &mut connection.receiver; 
                    let _ = receiver.send(msg).await;
                } 
            } 
        }

        /// Broadcast the raw binary bytes directly.
        pub async fn binary(&mut self, bytes: &Bytes) { 
            for connection in &mut self.connections { 
                let msg = Message::Binary(bytes.clone());
                let receiver = &mut connection.receiver; 
                                        
                let _ = receiver.send(msg).await;
            }
        }

        /// broadcast the raw binary bytes if the given condition in it's closure is true.
        pub async fn binary_if<F>(&mut self, bytes: &Bytes, condition: F) where F: Fn(&Connection) -> bool, { 
            for connection in &mut self.connections { 
                if condition(connection) { 
                    let msg = Message::Binary(bytes.clone());
                    let receiver = &mut connection.receiver; 
                    let _ = receiver.send(msg).await;
                } 
            } 
        }

        /// broadcast the raw binary bytes if the given condition in it's closure is false.
        pub async fn binary_if_not<F>(&mut self, bytes: &Bytes, condition: F) where F: Fn(&Connection) -> bool, { 
            for connection in &mut self.connections { 
                if !condition(connection) { 
                    let msg = Message::Binary(bytes.clone());
                    let receiver = &mut connection.receiver; 
                    let _ = receiver.send(msg).await;
                } 
            } 
        }

        /// Close all connections and remove it from it's room but not close it.
        pub async fn close(&mut self, close_frame: Option<CloseFrame>) { 
            self.connections.retain_mut(|connection| {
                let msg = Message::Close(close_frame.clone());
                let receiver = &mut connection.receiver; 
                                                
                let _ = async {
                    let _ = receiver.send(msg).await;
                };
                
                false
            });
        }

        /// it's most convenient way to close a single connection but keeping room open.
        pub async fn close_conn(&mut self, close_frame: Option<CloseFrame>, id: &String) {
            self.connections.retain_mut(|connection| {
                if connection.id == *id {
                    let msg = Message::Close(close_frame.clone());
                    let receiver = &mut connection.receiver; 
                                                    
                    let _ = async {
                        let _ = receiver.send(msg).await;
                    };
                    
                    false
                } else {
                    true
                }
            });
        }

        /// close each connection and remove them from room if the given condition in it's closure is true.
        pub async fn close_if<F>(&mut self, close_frame: Option<CloseFrame>, condition: F) where F: Fn(&Connection) -> bool, { 
            self.connections.retain_mut(|connection| {
                if condition(&connection) {
                    let msg = Message::Close(close_frame.clone());
                    let receiver = &mut connection.receiver; 
                                                    
                    let _ = async {
                        let _ =  receiver.send(msg).await;
                    };
                    
                    false
                } else {
                    true
                }
            });
        }

        /// close each connection and remove them from room if the given condition in it's closure is false.
        pub async fn close_if_not<F>(&mut self, close_frame: Option<CloseFrame>, condition: F) where F: Fn(&Connection) -> bool, { 
            self.connections.retain_mut(|connection| {
                if !condition(&connection) {
                    let msg = Message::Close(close_frame.clone());
                    let receiver = &mut connection.receiver; 
                                                            
                    let _ = async {
                        let _ =  receiver.send(msg).await;
                    };
                            
                    false
                } else {
                    true
                }
            });
        }
    }

    impl Broadcaster {
        /// create new broadcaster.
        pub fn new() -> Arc<RwLock<Self>> {
            Arc::new(RwLock::new(Self::default()))
        }

        /// get receiver and stream, similar to ".handle()" method of actix-ws.
        pub fn configure(socket: WebSocket) -> (SplitSink<WebSocket, Message>, SplitStream<WebSocket>) {
            socket.split()
        }

        /// handle the all thing. If you use that api, there is no need to any other configuration for grouping and identifying connections:
        pub async fn handle(broadcaster: &Arc<RwLock<Self>>, room_id: &String, conn_id: &String, receiver: SplitSink<WebSocket, Message>) -> Arc<RwLock<Self>> {
            let mut broadcaster_write = broadcaster.write().await;

            broadcaster_write.handle_room(room_id).add_connection(conn_id, receiver);

            Arc::clone(&broadcaster)
        }

        /// check if a room with given id exist and if it's not create one:
        pub fn handle_room(&mut self, id: &String) -> &mut Room {
            if let Some(index) = self.rooms.iter().position(|room| room.id == *id) {
                return &mut self.rooms[index];
            }
        
            self.rooms.push(Room {
                id: id.clone(),
                connections: vec![],
            });
        
            self.rooms.last_mut().unwrap()
        }

        /// Get the Room with given id. If there is a risk of unextistance of the room, use ".check_room()" instead.
        pub fn room(&mut self, id: &String) -> &mut Room {
            return self.rooms.iter_mut().find(|room| room.id == *id).unwrap();
        }

        /// check if a room with given id exist and wrap it in an option.
        pub fn check_room(&mut self, id: &String) -> Option<&mut Room> {
            match self.rooms.iter_mut().find(|room| room.id == *id) {
                Some(room) => Some(room),
                None => None
            }
        }

        /// only check if a room exist and if it's return true.
        pub fn check(&self, id: &String) -> bool {
            return self.rooms.iter().any(|room| room.id == *id);
        }

        /// it removes a room with given id and closes all the connections inside of it.
        pub async fn remove_room(&mut self, id: &String) {
            self.rooms.retain_mut(|room| { 
                if room.id == *id {
                    let _ = async {
                        let _ = room.close(None).await;
                    };
                    
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

        /// Removes the connection from Room. Warning: Because the async closures are not stable yet, we cannot close the connection in that function, you have to make cleanup on your cadebase. For that, check the examples & Documentation.
        pub fn remove_connection(&mut self, id: &String) -> Option<SplitSink<WebSocket, Message>> {
            for room in &mut self.rooms {
                if let Some(pos) = room.connections.iter().position(|connection| connection.id == *id) {
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