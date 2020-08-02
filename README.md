# Hey, that's my fish!

## Simple webapp version of a deceptively simple children's game

You can play a productionized Webassembly version of the game at
https://jthemphill.github.io/htmf

## How to run in webassembly mode

When running in Webassembly mode, the AI runs entirely locally on the client.
This is suitable for a "static" site like a Github page, where you're allowed
to ship HTML/CSS/JS to a client but aren't allowed to consume server resources.

You will need `cargo` and `npm`.

1. Go to `wasm` and run `npm run start`

## How to run in server mode

When running in server mode, a separate program runs in a loop and manages game
state for each client. It's suitable for cloud server instances where you can
consume resources.

Server mode is a little bit dated compared to Webassembly mode. I may delete it
soon.

You will need `cargo` and `yarn` .

1. In one terminal, go to `server` and run `cargo run` .
2. In another terminal, go to `client` and run `yarn run start` .
