import React from 'react'
import ReactDOM from 'react-dom/client'
import App from './App'
import type { BotWorker } from './WorkerProtocol'
import './index.css'

const container = document.getElementById('root')
if (container !== null) {
  const worker = new Worker(
    new URL('./bot.worker.ts', import.meta.url),
    { name: 'Rules engine and AI', type: 'module' }
  ) as BotWorker
  ReactDOM.createRoot(container).render(
    <React.StrictMode>
      <App worker={worker} />
    </React.StrictMode>
  )
}
