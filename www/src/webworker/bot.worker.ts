import htmfWasmInit, * as htmfWasm from "htmf-wasm";
import type { WorkerRequest } from "../browser/WorkerProtocol";
import Bot from "./Bot";

// Run WebAssembly.instantiateStreaming() to load and initialize the WebAssembly module
const memory = new WebAssembly.Memory({
  initial: 18,
  maximum: 16384,
  shared: true,
});
const wasmInternals = await htmfWasmInit({ memory });

// Initialize a pool with one WebWorker per available core
await htmfWasm.initThreadPool(navigator.hardwareConcurrency);

const bot = new Bot(wasmInternals, postMessage);
onmessage = (event: MessageEvent<WorkerRequest>) => {
  bot.onMessage(event.data);
};

export {};
