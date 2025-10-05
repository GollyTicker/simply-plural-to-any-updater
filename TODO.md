# TODO

Add a small additional binary to the `src` rust crate, which opens up a websocket to Simply Plural and listens for friend requenst events.

For each received friend request, it simply accepts it.

We don't know the format in which we receive the friend request events. So we'll need to first create and listen via the websocket and then we can see what format we get.

The documentation of how it all works is written below in the documentation section.

## STEPS

1. DONE. Create a new binary within the `src` rust crate.
2. DONE. Establish a WebSocket connection to the Simply Plural API at `wss://api.apparyllis.com/v1/socket`.
3. DONE. Authenticate the connection by sending an `{'op': 'authenticate', 'token': "your-token"}` payload.
4. DONE. Implement a keep-alive mechanism, sending a "ping" message every 10 seconds to maintain the connection.
5. DONE. Listen for incoming WebSocket messages to determine the structure of friend request events.
6. DONE. Upon receiving a friend request, fetch all friend requests from simply plural and accept all of them one by one.

---

## Documentation from Simply Plural Website

### Socket connection
Connecting

To create a socket connection please connect to either of the below Urls:

** Production **
wss://api.apparyllis.com/v1/socket

** Pretesting **
wss://devapi.apparyllis.com/v1/socket
Authenticating

Once connected to the socket you will need to authenticate, you can do this by sending this payload

{'op': 'authenticate', 'token': "insert-your-token-here"}

You will receive one of two messages:

Authentication violation: Token is missing or invalid. Goodbye :)

or

Successfully authenticated

If you receive the violation make sure your token is valid and that there's no extra characters or white space at the front or end of the token. Create a new connection and re-authenticate. You can't try to authenticate more than once on the same connection.

Otherwise if there's no violation you are fully authenticated and you will receive updates when your authenticated user has updates.
Keeping alive

You will need to keep the connection alive to make sure it doesn't drop.
To do so simply send the following string to the socket every 10 seconds:

ping

Data format

When an insert, update or delete happens on one of your authenticated users' data you will be notified in the following format:
payload

{
    "msg" : "Update",
    "target" : "collection (members, frontHistory, etc.)",
    "results" : [
        {
            "operationType" : "delete/insert/update",
            "id" : "Id of the object that changed",
            "content" : { ... }
        }
    ]
}

### Get Received friend requests
Get received friend requests

GET 
https://api.apparyllis.com/v1/friends/requests/incoming

Get incoming friend requests of the authenticated user

Example response:

[
  {
    "exists": true,
    "id": "124cab14acb23c4b2ca34",
    "content": {
      "message": "",
      "username": "Ayake-System",
      "uid": "124cab14acb23c4b2ca34"
    }
  }
]

Each array element contains a single friend request.

### Accept friend request

Example:
curl -v -X POST -H "Content-Type: application/json" -H "Authorization: <token>" \
  https://api.apparyllis.com/v1/friends/request/respond/124cab14acb23c4b2ca34?accepted=true \
  --data-raw '{"settings":{"seeMembers": false, "seeFront": false, "getFrontNotif": false}}'