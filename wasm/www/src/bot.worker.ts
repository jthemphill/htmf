import Bot from './Bot'
import __wbg_init from 'htmf-wasm'

// Run WebAssembly.instantiateStreaming() to load and initialize the WebAssembly module
await __wbg_init()

const bot = new Bot(postMessage)
onmessage = (event) => {
  bot.onMessage(event.data)
}
bot.init()

export { }
