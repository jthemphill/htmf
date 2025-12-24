import htmfWasmInit from "htmf-wasm";
import Bot from "./Bot";
// Run WebAssembly.instantiateStreaming() to load and initialize the WebAssembly module
const wasmInternals = await htmfWasmInit();
const bot = new Bot(wasmInternals, postMessage);
onmessage = (event) => {
    bot.onMessage(event.data);
};
