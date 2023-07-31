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
  server: {
    fs: { allow: [searchForWorkspaceRoot(process.cwd()), '../pkg'] }
  },
  worker: {
    format: 'es'
  }
})
