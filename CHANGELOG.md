# CHANGELOG

## v0.11.0

- added `.pong()`, `.pong_if()` and `.pong_if_not()` methods to `Connection` type of both implementations.

## v0.10.0

- added `.ping()`, `.ping_if()` and `.ping_if_not()` methods to `Connection` type of both implementations.
- axum version updated to `v0.8.4`

## v0.9.0

- added `.send()`, `.send_if()` and `.send_if_not()` methods to `Connection` type of both implementations.

## v0.8.0

- added `.each_room_immut()`, `.each_room()` and `.each_room_mut()` methods to the `Broadcaster` type of both implementations.
- upgraded `tokio`, `serde` and `serde_json` crates versions to their latests.

## v0.7.0

- added `.close_conn()` method to `Room` struct of both implementations. It benefits them to close single connection with given id and not closes rooms.

## v0.6.0

- typed and normal api's are changed, now they uses pointers of arguments most of the time rather than using actual arguments.

## v0.5.1

- Compatibility update. Normal api's Axum version updated to `v0.8.1` and made some compatibility updates. Typed api still uses axum `v0.7.9`.

## v0.5.0

- Added `.close()`, `.close_if()` and `.close_if_not()` methods on the `Room` struct of both implementations. They closes the connection and removes it from it's room but they don't close the room.
- Also added `.remove_room()` method for `Broadcaster` struct of both implementations. It removes the room with given id and closes all it's connections.

## v0.4.0

- Added `.binary()`, `.binary_if()` and `.binary_if_not()` method for normal api.

## v0.3.0

- Added `.pong()`, `.pong_if()` and `.pong_if_not()` method on both implementations.

## v0.2.0

- Changelog file created.
- Added `.ping()`, `.ping_if()` and `.ping_if_not()` method on both implementations.

## v0.1.0

- Liblary Created.
