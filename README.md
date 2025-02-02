# Axum Websocket Broadcaster

A broadcasting liblary for both [axum-typed-websockets](https://crates.io/crates/axum-typed-websockets) and `axum::extract::ws`, similar to [actix-ws-broadcaster](https://crates.io/crates/actix-ws-broadcaster).

This liblary is basically the equivalent of [actix-ws-broadcaster](https://crates.io/crates/actix-ws-broadcaster), but for axum ecosystem. Most of the api's work the same way and their usage is almost identical with some of the exceptions.

This liblary provides grouping and broadcasting mechanism for both websocket implementations of axum ecosystem. You have individual `Connection` for each "Receiver"s, will be identified as the given id. And there is also rooms exist, which benefits to group related connections on a single entity.

## Guide

### Adding dependency

Add that to your `Cargo.toml` file:

```toml

axum-ws-broadcaster = "0.6.0"

# Or:

axum-ws-broadcaster = { version = "0.6.0", features = ["typed"] }

```

### Import

```rust

use axum_wsb::normal::Broadcaster;

// Or:

use axum_wsb::typed::Broadcaster;

```

### Initialize

Initialize it in a place which it can hold it's state:

```rust

let receivers: Arc<RwLock<Broadcaster>> = Broadcaster::new();

// Or if you use typed broadcaster, define the types. First type represents which type do we send to the receivers and the second type is the type which we receive from senders.

let receivers: Arc<RwLock<Broadcaster<T, S>>> = Broadcaster::new();

```

### Handle Connections And Rooms

We implemented a `configure()` function, which takes WebSocket as argument and returns the `receiver` and `stream`:

```rust

let (receiver, mut stream) = Broadcaster::configure(socket);

```

Later you have to handle the connections and rooms in the websocket route:

```rust

// first argument is the broadcaster instance
// second argument is the id of room which we want to put the connection
// third argument is the which we want to assign to connection
// fourth argument the receiver of connection.

let broadcaster = Broadcaster::handle(&broadcaster, &room_id, &conn_id, receiver).await;

```

They work both same on two api's.

### Broadcast The Messages

Note: You have to do broadcasting in same broadcaster instance, don't clone it. Otherwise it could cause data race.

The typed and normal api's works slightly differently, follow the guide:

In the loop of websocket, if a message received, you can broadcast it by that code:

#### Normal

If you are familiar, normal api works almost identical to the websockets of [actix-ws-broadcaster](https://crates.io/crates/actix-ws-broadcaster):

```rust

Message::Text(input) => {
    let mut broadcaster = broadcaster.write().await;

    let _ = broadcaster.room(&query.room).broadcast(&input).await;
}

```

#### Typed

But typed websockets is more different due to they are "typed":

```rust

Message::Item(input) => {
    // the "input" in the Item variant of Message is turned into the type of second
    // generic type that you provide on broadcaster. Do what you want with it.
    let input = input;

    let mut broadcaster = broadcaster.write().await;

    // and create your output as the first generic type that you provided.
    // .broadcast() is will convert that type into the valid websocket message
    // and send to the client.
    let output = output;

    // than perform the broadcasting:
    let _ = broadcaster.room(&query.room).broadcast(&output).await;
},

```

### A Comprehensive example

#### Normal Api

```rust

async fn websocket_handler(ws: WebSocketUpgrade, Query(query): Query<WebsocketQueries>, State(state): State<Arc<RwLock<Broadcaster>>>) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, Query(query), state))
}


async fn handle_socket(socket: WebSocket, Query(query): Query<WebsocketQueries>, state: Arc<RwLock<Broadcaster>>) {
    let (receiver, mut stream) = Broadcaster::configure(socket);

    let broadcaster = Broadcaster::handle(&state, &query.room, &query.id, receiver).await;

    while let Some(msg_result) = stream.next().await {
        match msg_result {
            Ok(message) => {
                match message {
                    Message::Text(input) => {
                        let mut broadcaster = broadcaster.write().await;

                        let _ = broadcaster.room(&query.room).broadcast(&input).await;
                    },
                    Message::Close(_) => {
                        // this is the old way of closing connections and making cleanup:

                        let mut broadcaster = broadcaster.write().await;

                        let _ = broadcaster.remove_connection(&query.id).unwrap().close().await;

                        // the new way. This removes all the connections but keeps room open:
                        let mut broadcaster = broadcaster.write().await;

                        let _ = broadcaster.room(&query.room).close(None).await;

                        // this is the most proper way if you want to fully close a room:

                        let mut broadcaster = broadcaster.write().await;
                        
                        let _ = broadcaster.remove_room(&query.room).await;

                        return;
                    },
                    Message::Ping(ping) => {
                        let mut broadcaster = broadcaster.write().await;

                        let _ = broadcaster.room(&query.room).pong(&ping).await;
                    },
                    Message::Pong(pong) => {
                        let mut broadcaster = broadcaster.write().await;

                        let _ = broadcaster.room(&query.room).ping(&pong).await;
                    },
                    Message::Binary(binary) => {
                        let mut broadcaster = broadcaster.write().await;

                        let _ = broadcaster.room(&query.room).binary(&binary).await;
                    }
                }
            },
            Err(error) => println!("that error occured: {}", error)
        }
    }
}

```

#### Typed Api

Assuming you passed broadcaster as a `State()`, your sending type is `String`, receiving type is `WeboscketInput` and they have same fields:

```rust

async fn websocket_handler(ws: WebSocketUpgrade<String, WebsocketInput>, Query(query): Query<WebsocketQueries>, State(state): State<Arc<RwLock<Broadcaster<String, WebsocketInput>>>>) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, Query(query), state))
}

async fn handle_socket(socket: WebSocket<String, WebsocketInput>, Query(query): Query<WebsocketQueries>, state: Arc<RwLock<Broadcaster<String, WebsocketInput>>>) {
    let (receiver, mut stream) = Broadcaster::configure(socket);

    let broadcaster = Broadcaster::handle(&state, &query.room, &query.id, receiver).await;

    while let Some(msg_result) = stream.next().await {
        match msg_result {
            Ok(message) => {
                match message {
                    Message::Item(input) => {
                        let output = WebsocketOutput {
                            name: input.name,
                            id: input.id,
                            message: input.message
                        };

                        // since our first generic type is string, we have to send
                        // something that has the type of string. On this example,
                        // we convert our output to json and send it by that:
                        let output = serde_json::to_string(&output).unwrap();

                        let mut broadcaster = broadcaster.write().await;

                        let _ = broadcaster.room(&query.room).broadcast(&output).await;
                    },
                    Message::Close(_) => {
                        // this is the old way of closing connections and making cleanup:
                        let mut broadcaster = broadcaster.write().await;

                        let _ = broadcaster.remove_connection(&query.id).unwrap().close().await;

                        // the new way. This removes all the connections but keeps room open:
                        let mut broadcaster = broadcaster.write().await;

                        let _ = broadcaster.room(&query.room).close(None).await;

                        // this is the most proper way if you want to fully close a room:

                        let mut broadcaster = broadcaster.write().await;
                        
                        let _ = broadcaster.remove_room(&query.room).await;
                        
                        return;
                    },
                    Message::Ping(ping) => {
                        let mut broadcaster = broadcaster.write().await;

                        let _ = broadcaster.room(&query.room).pong(&ping).await;
                    },
                    Message::Pong(pong) => {
                        let mut broadcaster = broadcaster.write().await;

                        let _ = broadcaster.room(&query.room).ping(&pong).await;
                    }
                }
            },
            Err(error) => println!("that error occured: {}", error)
        }
    }
}

```

## Try It Yourself

To try it yourself, run that commands:

`cargo run --example normal-example`

Or:

`cargo run --example typed-example --features typed`

Than go to the `http://localhost:5000` address on a firefox based browser(such as firefox, librewolf etc.). Because chromium based browsers don't support to send query parameters to websockets from the javascript, our front-end configuration don't work on them. In real world scenarios, you have to provide room and connection id's with different approach.

## Contribution Guide

Issues, suggestions and pull requests are welcome.
