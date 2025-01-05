use axum_wsb::normal::Broadcaster;
use std::{fmt::Display, sync::Arc};
use axum_8_1::{Router, response::{Response, IntoResponse}, routing::get, extract::{State, Query, ws::{WebSocket, WebSocketUpgrade, Message}}};
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use futures_util::StreamExt;

#[tokio::main]
async fn main() {
    let receivers: Arc<RwLock<Broadcaster>> = Broadcaster::new();
    
    let router = Router::new()
                                .route("/", get(home_controller().await))
                                .route("/chat", get(chat_controller().await))
                                .route("/chats", get(websocket_handler))
                                .with_state(receivers);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:5000").await.unwrap();
    
    axum_8_1::serve(listener, router).await.unwrap();
}

pub async fn home_controller() -> Response<String> {
    let html = "<!DOCTYPE html>
                        <html lang='en'>
                            <head>
                                <meta charset='UTF-8'>
                                <meta name='viewport' content='width=device-width, initial-scale=1.0'>
                                <title>Document</title>
                            </head>
                            <body>
                                <h1>Hello!</h1>

                                <form action='/chat' method='get'>
                                    <input type='text' name='room' placeholder='room'>
                                    <input type='text' name='name' placeholder='name'>
                                    <input type='text' name='id' placeholder='id'>
                                    <input type='submit' value='send'>
                                </form>
                            </body>
                        </html>".to_string();

    Response::builder()
             .header("Content-Type", "text/html")
             .body(html)
             .unwrap()
}

pub async fn chat_controller() -> Response<String> {
    let html = "<!DOCTYPE html>
                        <html lang='en'>
                        <head>
                            <meta charset='UTF-8'>
                            <meta name='viewport' content='width=device-width, initial-scale=1.0'>
                            <title>Document</title>

                        </head>
                        <body>
                            <input type='message' placeholder='send chat' class='message-input'>
                            <input type='submit' value='send' class='send-chat-button'>
                            <button class='close-button'>close</button>
                            
                            <div class='chats'>

                            </div>

                            <style>
                                .messages {
                                    min-width: 100px;
                                    height: 50px;
                                    color: white;
                                    margin: 10px 0;
                                    border-radius: 20px;
                                }

                                .my-message {
                                    background-color: black;
                                }

                                .other-message {
                                    background-color: blue;
                                }
                            </style>

                        <script> 
                            const chats = document.querySelector('.chats'); 
                            const send = document.querySelector('.send-chat-button'); 
                            const messageInput = document.querySelector('.message-input');  
                            const query = new URLSearchParams(window.location.search); 
                            const closeButton = document.querySelector('.close-button');

                            /* Note: That configuration doesn't work on chromium based browsers,
                            Because they don't let you to send query parameters to websocket 
                            routes with Websocket Api. You should try it on firefox based 
                            browsers, such as firefox, librewolf etc. */
                            
                            let websocketUrl = `ws://localhost:5000/chats?name=${query.get('name')}&id=${query.get('id')}&room=${query.get('room')}`

                            let websocket = new WebSocket(websocketUrl);

                            websocket.addEventListener('open', function() { 
                                console.log('WebSocket is open!'); 
                            }); 

                            websocket.addEventListener('message', async function(event) {
                                const message = JSON.parse(event.data); 
                                const newParagraph = document.createElement('p'); 
                                newParagraph.textContent = message.name + ': ' + message.message; 
                                newParagraph.classList.add('messages'); 
                                
                                if (message.id === query.get('id')) { 
                                    newParagraph.classList.add('my-message'); 
                                } else { 
                                    newParagraph.classList.add('other-message'); 
                                } 
                                
                                chats.append(newParagraph);
                            }); 
                            
                            websocket.addEventListener('close', function(event) { 
                                console.log('WebSocket closed: ', event); 
                                console.log('is event bubbled: ', event.bubbles);
                                console.log('is event composed: ', event.composed);
                                console.log(`Code: ${event.code}, Reason: ${event.reason}`); 
                            }); 
                            
                            websocket.addEventListener('error', function(event) { 
                                console.error('WebSocket error: ', event); 
                            }); 

                            document.addEventListener('beforeunload', function(){
                                websocket.close();
                            })
                            
                            send.addEventListener('click', function() { 
                                const message = { 
                                    name: query.get('name'), 
                                    id: query.get('id'), 
                                    message: messageInput.value 
                                }; 
                                
                                websocket.send(JSON.stringify(message)); 
                                
                                messageInput.value = ''; 
                            }); 

                            closeButton.addEventListener('pointerdown', function() { 
                                websocket.close();
                            }); 
                        </script>
                        </body>
                        </html>".to_string();

    Response::builder()
             .header("Content-Type", "text/html")
             .body(html)
             .unwrap()
}



async fn websocket_handler(ws: WebSocketUpgrade, Query(query): Query<WebsocketQueries>, State(state): State<Arc<RwLock<Broadcaster>>>) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, Query(query), state))
}


async fn handle_socket(socket: WebSocket, Query(query): Query<WebsocketQueries>, state: Arc<RwLock<Broadcaster>>) {
    let (receiver, mut stream) = Broadcaster::configure(socket);

    let broadcaster = Broadcaster::handle(&state, query.room.clone(), query.id.clone(), receiver).await;

    while let Some(msg_result) = stream.next().await {
        match msg_result {
            Ok(message) => {
                match message {
                    Message::Text(input) => {
                        let mut broadcaster = broadcaster.write().await;

                        let _ = broadcaster.room(query.room.clone()).broadcast(input).await;
                    },
                    Message::Close(_) => {
                        // this is the old way of closing connections and making cleanup:

                        /*
                        let mut broadcaster = broadcaster.write().await;

                        let _ = broadcaster.remove_connection(query.id).unwrap().close().await;
                        */

                        // the new way. This removes all the connections but keeps room open:
                        /*let mut broadcaster = broadcaster.write().await;

                        let _ = broadcaster.room(query.room).close(None).await;*/

                        // this is the most proper way if you want to fully close a room:

                        let mut broadcaster = broadcaster.write().await;
                        
                        let _ = broadcaster.remove_room(query.room).await;

                        return;
                    },
                    Message::Ping(ping) => {
                        let mut broadcaster = broadcaster.write().await;

                        let _ = broadcaster.room(query.room.clone()).pong(ping).await;
                    },
                    Message::Pong(pong) => {
                        let mut broadcaster = broadcaster.write().await;

                        let _ = broadcaster.room(query.room.clone()).ping(pong).await;
                    },
                    Message::Binary(binary) => {
                        let mut broadcaster = broadcaster.write().await;

                        let _ = broadcaster.room(query.room.clone()).binary(binary).await;
                    }
                }
            },
            Err(error) => println!("that error occured: {}", error)
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WebsocketOutput {
    name: String,
    id: String,
    message: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WebsocketInput {
    name: String,
    id: String,
    message: String
}

impl Display for WebsocketInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self) // Debug çıktısını kullanıyoruz
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WebsocketQueries {
    pub name: String,
    pub room: String,
    pub id: String
}
