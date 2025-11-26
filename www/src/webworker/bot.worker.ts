import htmfWasmInit from "htmf-wasm";
import type { WorkerRequest } from "../browser/WorkerProtocol";
import Bot from "./Bot";

// Run WebAssembly.instantiateStreaming() to load and initialize the WebAssembly module
const wasmInternals = await htmfWasmInit();

const bot = new Bot(wasmInternals, postMessage);
onmessage = (event: MessageEvent<WorkerRequest>) => {
  bot.onMessage(event.data);
};

export {};
