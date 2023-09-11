import Bot from './Bot'
import htmfWasmInit from 'htmf-wasm'

// Run WebAssembly.instantiateStreaming() to load and initialize the WebAssembly module
const wasmInternals = await htmfWasmInit()

const bot = new Bot(wasmInternals, postMessage)
onmessage = (event) => {
  bot.onMessage(event.data)
}
bot.postGameState({ })

export { }
