# Hey, that's my fish!

## Simple webapp version of a deceptively simple children's game

You can play a productionized Webassembly version of the game at
https://jthemphill.github.io/htmf

## How to run in Webassembly mode

When running in Webassembly mode, the AI runs entirely locally on the client.
This is suitable for a "static" site like a Github page, where you're allowed
to ship HTML/CSS/JS to a client but aren't allowed to consume server resources.

You will need `cargo` and `npm`. Go to `wasm/www` and run `npm run start`.
