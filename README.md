## How to run
`cargo run -- --port <port number>`

At the moment, only ports 9001, 9002, and 9003 are acceptable ports.

### Future improvements
For resiliency (server timeouts, dropped messages, etc.) consider leveraging an offbox queue to store incoming messages before relaying them.  With the additional step of placing messages to be relayed into a message queue, messages will no longer be dropped if a server to which a message must be relayed goes offline temporarily.  A retry policy can be set on messages in the queue so that they are requeued in the event of a failed attempt for as many times or as long as their transmission is needed.