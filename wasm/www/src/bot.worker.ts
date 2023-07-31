import Bot from './Bot'
import htmfWasmInit from 'htmf-wasm'

// Run WebAssembly.instantiateStreaming() to load and initialize the WebAssembly module
await htmfWasmInit()

const bot = new Bot(postMessage)
onmessage = (event) => {
  bot.onMessage(event.data)
}
bot.init()

export { }
