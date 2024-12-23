#[cfg(feature = "typed")]
use axum_wsb::typed::Broadcaster;
#[cfg(feature = "typed")]
use std::{fmt::Display, sync::Arc};
#[cfg(feature = "typed")]
use axum_typed_websockets::{WebSocket, WebSocketUpgrade, Message};
#[cfg(feature = "typed")]
use axum::{Router, response::{Response, IntoResponse}, routing::get, extract::{State, Query}};
#[cfg(feature = "typed")]
use tokio::sync::RwLock;
#[cfg(feature = "typed")]
use serde::{Serialize, Deserialize};
#[cfg(feature = "typed")]
use serde_json;
#[cfg(feature = "typed")]
use futures_util::{SinkExt, StreamExt, stream::{SplitStream, SplitSink}};

#[tokio::main]
async fn main() {
    #[cfg(feature = "typed")]
    let receivers: Arc<RwLock<Broadcaster<String, WebsocketInput>>> = Broadcaster::new();

    #[cfg(feature = "typed")]
    let router = Router::new()
                                .route("/", get(home_controller().await))
                                .route("/chat", get(chat_controller().await))
                                .route("/chats", get(websocket_handler))
                                .with_state(receivers);

    #[cfg(feature = "typed")]
    let listener = tokio::net::TcpListener::bind("127.0.0.1:5000").await.unwrap();
    
    #[cfg(feature = "typed")]
    axum::serve(listener, router).await.unwrap();
}

#[cfg(feature = "typed")]
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

#[cfg(feature = "typed")]
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
                                if (event.data instanceof Blob) {
                                    const text = await event.data.text();
                                    console.log('Received text message:', text);

                                    const message = JSON.parse(JSON.parse(text)); 
                                    console.log('converted message: ', message);
                                    const newParagraph = document.createElement('p'); 
                                    newParagraph.textContent = message.name + ': ' + message.message; 
                                    newParagraph.classList.add('messages'); 
                                    
                                    if (message.id === query.get('id')) { 
                                        newParagraph.classList.add('my-message'); 
                                    } else { 
                                        newParagraph.classList.add('other-message'); 
                                    } 
                                    
                                    chats.append(newParagraph);
                                }

                                //console.log('işte gelen mesaj: ', event.data);
                                /*const message = JSON.parse(event.data); 
                                const newParagraph = document.createElement('p'); 
                                newParagraph.textContent = message.name + ': ' + message.message; 
                                newParagraph.classList.add('messages'); 
                                
                                if (message.id === query.get('id')) { 
                                    newParagraph.classList.add('my-message'); 
                                } else { 
                                    newParagraph.classList.add('other-message'); 
                                } 
                                
                                chats.append(newParagraph);*/
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
                        </script>
                        </body>
                        </html>".to_string();

    Response::builder()
             .header("Content-Type", "text/html")
             .body(html)
             .unwrap()
}


#[cfg(feature = "typed")]
async fn websocket_handler(ws: WebSocketUpgrade<String, WebsocketInput>, Query(query): Query<WebsocketQueries>, State(state): State<Arc<RwLock<Broadcaster<String, WebsocketInput>>>>) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, Query(query), state))
}

#[cfg(feature = "typed")]
async fn handle_socket(socket: WebSocket<String, WebsocketInput>, Query(query): Query<WebsocketQueries>, state: Arc<RwLock<Broadcaster<String, WebsocketInput>>>) {
    use axum::extract::ws::Message;

    let (receiver, mut stream) = Broadcaster::configure(socket);

    let broadcaster = Broadcaster::handle(&state, query.room.clone(), query.id.clone(), receiver).await;

    while let Some(msg_result) = stream.next().await {
        match msg_result {
            Ok(message) => {
                match message {
                    axum_typed_websockets::Message::Item(input) => {
                        let output = WebsocketOutput {
                            name: input.name,
                            id: input.id,
                            message: input.message
                        };

                        let mut broadcaster = broadcaster.write().await;

                        let output = serde_json::to_string(&output).unwrap();

                        let _ = broadcaster.room(query.room.clone()).broadcast(output).await;
                    },
                    axum_typed_websockets::Message::Close(_) => {
                        let mut broadcaster = broadcaster.write().await;

                        let _ = broadcaster.remove_connection(query.id).unwrap().close().await;
                        
                        return;
                    },
                    axum_typed_websockets::Message::Ping(ping) => {
                        let mut broadcaster = broadcaster.write().await;

                        let _ = broadcaster.room(query.room.clone()).pong(ping).await;
                    },
                    axum_typed_websockets::Message::Pong(pong) => {
                        let mut broadcaster = broadcaster.write().await;

                        let _ = broadcaster.room(query.room.clone()).ping(pong).await;
                    }
                }
            },
            Err(error) => println!("that error occured: {}", error)
        }
    }
}

#[cfg(feature = "typed")]
#[derive(Debug, Serialize, Deserialize)]
pub struct WebsocketOutput {
    name: String,
    id: String,
    message: String
}

#[cfg(feature = "typed")]
#[derive(Debug, Serialize, Deserialize)]
pub struct WebsocketInput {
    name: String,
    id: String,
    message: String
}

#[cfg(feature = "typed")]
impl Display for WebsocketInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self) // Debug çıktısını kullanıyoruz
    }
}

#[cfg(feature = "typed")]
#[derive(Debug, Serialize, Deserialize)]
pub struct WebsocketQueries {
    pub name: String,
    pub room: String,
    pub id: String
}
