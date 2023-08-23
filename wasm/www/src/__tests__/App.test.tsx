import { render, screen } from '@testing-library/react'
import { describe, it, expect } from 'vitest'

import { NUM_CELLS } from '../constants'
import App from '../App'
import { type AppWorker } from '../App'
import { type WorkerRequest, type WorkerResponse } from '../WorkerProtocol'
import Bot from '../Bot'
import htmfWasmInit from '../../../pkg/htmf_wasm'

import fs from 'fs'
import path from 'node:path'

async function loadHtmfWasm (): Promise<ArrayBuffer> {
  const buf = await fs.promises.readFile(path.resolve(__dirname, '../../../pkg/htmf_wasm_bg.wasm'))
  return buf.buffer
}

class MockWorker implements AppWorker {
  postMessage: (request: WorkerRequest) => void
  onmessage: (this: Worker, ev: MessageEvent<WorkerResponse>) => any
  onmessageerror: null

  constructor () {
    this.postMessage = () => { }
    this.onmessage = () => { }
    this.onmessageerror = null
  }

  terminate (): void { }
  addEventListener (): void { }
  removeEventListener (): void { }
  onerror (): void { }
  dispatchEvent (): boolean { return false }
}

describe('App', async () => {
  const wasmInternals = await htmfWasmInit(loadHtmfWasm())

  const mockWorker = new MockWorker()
  const bot = new Bot(wasmInternals, (response: WorkerResponse) => {
    if (mockWorker.onmessage !== null) {
      mockWorker.onmessage(new MessageEvent<WorkerResponse>('message', { data: response }))
    } else {
      console.error("Mock WebWorker's onmessage property hasn't been initialized yet.")
    }
  })
  mockWorker.postMessage = (request: WorkerRequest) => {
    bot.onMessage(request)
  }

  it('renders a game board', async () => {
    render(<App worker={mockWorker} />)
    bot.init()

    const buttons = (await screen.findAllByRole('button'))

    expect(buttons.filter(btn => btn.classList.contains('cell')).length).toEqual(NUM_CELLS)
  })
})
