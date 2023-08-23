/// <reference types="vitest" />

import { defineConfig, searchForWorkspaceRoot } from 'vite'
import react from '@vitejs/plugin-react-swc'

// https://vitejs.dev/config/
export default defineConfig({
  base: './',
  build: {
    target: 'esnext',
    sourcemap: true
  },
  plugins: [react()],
  resolve: { dedupe: ["react", "react-dom"] },
  server: {
    fs: { allow: [searchForWorkspaceRoot(process.cwd()), '../pkg'] }
  },
  test: {
    environment: 'jsdom',
    setupFiles: 'setup-vitest.ts',
  },
  worker: {
    format: 'es'
  }
})
