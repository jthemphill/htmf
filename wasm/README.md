# Hey, that's my Fish!
## Webassembly edition

This is a wrapper for *Hey, that's my Fish!* designed to run in the browser.

## How to run in Webassembly mode

When running in Webassembly mode, the AI runs entirely locally on the client.
This is suitable for a "static" site like a Github page, where you're allowed to
ship HTML/CSS/JS to a client but aren't allowed to consume server resources.

WebAssembly mode utilizes WebAssembly threads,
[which have not been standardized yet](https://rustwasm.github.io/wasm-bindgen/examples/raytrace.html).

So for now, you'll need a nightly build of Rust:

```
% rustup install nightly
```

Lastly, you will need `npm` in order to build this game's TypeScript.

Once you have a nightly build of Rust and a build of `npm`:

1. Go to `wasm/www`.
2. Run `npm run build:wasm-st` to build the WebAssembly.
3. Run `npm run dev` to start the Vite devserver.
