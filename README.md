# What this is
Dockerized app that makes a web app and api to serve a chat that goes trough websockets
## TODO:
- Change message type sent to have a format like {"type": "broadcast", "metadata":{"session_id":"<sess_id>","sent_when_override":"00_00_000"},"content":"i just use arch"}
- Record who is in which websocket sender (save user_id, senders should have a list of their ids)
- Support "private" messages (never truly private but not really anyone will see them just the server you connected to)
  - Support "Who" messages to know and verify who is in which sender
- Make an fukk api helper so implementation is easier
- Support /api/me?**id** and /api/logout?**id** parameters

## Temporal api help
### Endpoints: 
> /api/ws -> websocket connection
> /api/me?id=<sessid> -> verify wether session is expired, default takes cookie as session_id, in the future it will support id get parameter (400 OK = not expired)
> /api/login -> returns cookie with session_id
  POST: Login_Request
> /api/register -> returns cookie with session_id and errors if user already exists
  POST: Login_Request
> /api/logout?id=<sessid> -> errases cookie and closes session given by cookie or id parameter in the future
### data_structures
