Promise.all([
  import("./Bot"),
  // Run WebAssembly.instantiateStreaming() to load and initialize the WebAssembly module
  import("htmf-wasm").then(async (wasm) => await wasm.default()),
]).then(
  ([Bot, wasm]) => {
    const bot = new Bot.default(postMessage);
    onmessage = (event) => {
      bot.onMessage(event.data);
    };
    bot.init();
  },
);

export { };