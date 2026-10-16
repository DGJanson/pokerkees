## Pokerkees

Minimal TUI implementation of Azure Devops Poker.

Consists of two parts: server and client. The server stores shared poker status. Several clients can access the server and perform poker sessions. The flow is as follows:

User clients connect to server. The servers holds several sessions. A session consists of several items. Users can add sessions, items as they like. Pokerkees is meant to be used in conjunction with a communication app (or face to face meeting) and an environment for managing backlog, like devops or jira.

Note there is NO SECURITY IMPLEMENTED. There is no authentication, authorisation, etc. The server should not have important data; it only holds session names, item numbers and votes.

### Poker server

The server is technically multithreaded, async. But in reality it handles requests synchronized. ALL info is stored in a Mutexed Hashmap. But the operations are so simple that performance should not be an issue for a few handfuls of users.

The server does NOT:

- have any security
- support large workloads
- persist any data, everything stays in memory

### Poker client

The poker client is a simple TUI. It is completely keyboard driven. Most key-inputs are shown in the UI. General shortcuts:

- esc: return to main screen
- j / k: navigate down / up
- enter: select / confirm
- ctrl + q: exit app

To setup the app you can provide username, server url as parms while booting OR in the app.

The poker client has a worker thread that syncs with server. It is a single, synced worker that performs IO based on an event queue managed by the client.

#### Azure integration

The client supports azure devops integration -> it can read work item info from your boards. Note that this sync is only done client-side; no azure data ever reaches the server.

Sync is performed using the azure cli. Specifically, the following command is performed:

```
az boards work-item show --id <id> --org <devops_org> 
```

If you can perform this command in your terminal (it may need some setting up), then pokerkees can sync with devops. For that you need to set Azure Organisation AND Azure Project in Pokerkees, either with cli arguments or in-app settings. You MUST use the same pokerkees item ids as azure work item ids. Then it will sync.

## TODO list

- [x] item handling
- [ ] show finished items
- [x] enrich with devops data (az boards command)
- [ ] try to make work item description and acceptance criteria bit more readable
- [ ] fetch list of items from azure to add to pokerkees (eg based on tag + area)
- [ ] gracefully handle deleted session
