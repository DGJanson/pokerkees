# Pokerkees

This is a small server - TUI application for pokering backlog items. Eg, during an agile refinement, the developers will "score" tasks / user items / etc for effort or story points. This can be done by "pokering". All developers will secretly enter their estimates, after which all estimates are revealed and a consensus can be reached.

## Data flow

The data is a simple hierarchical structure. The base is a session. A session holds items. Items can be voted on. Users can see all sessions and items. Hence the data should be simple and as little as possible.

## App setup

There are 3 components of this app. Note they are seperate crates in a single project.

- shared-lib: basically type definitions of the common data objects
- server: the server holding the state, that data, of the poker sessions
- client: a simple TUI app for interacting with poker sessions

## Server

The server is extremely simple. There is no authentication or authorization, it just holds numbers. No data except poker data should be present on the server.

State is kept in a single mutexed hashmap. All interactions with the state from clients should be **as simple and quick** as possible.

Since poker sessions are quite short and generally not super important, no persistence is needed. Everything is kept in memory, and only in memory.

The server should responds either with json data or a simple status code. Since the client and server "speak the same language" a simple status code will suffice; we do NOT need elaborate error handling.

## Client

The client is made with ratatui. It should be a very simply UI, no bells and whistles; simply a way to get and send info to the Server.

We use CLI arguments to preset some config options -> the server url and the username.

The client comes with a **single** worker thread to interact with the server. The state holds a queue to which WorkerRequests can be added. The worker polls for events one at a time and performs them, sending a WorkerEvent. There is an Error WorkerEvent to communicate errors.

Note that whilst the TUI is active (which is most of the time), we do NOT log. Logging is for errors during startup and exiting. Errors can be communicated using the AppState. There is no room for fatal errors yet, but this could be created if necessary.
