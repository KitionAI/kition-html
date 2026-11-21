import { fileURLToPath } from 'node:url'
import react from '@vitejs/plugin-react'
import { defineConfig } from 'vite'

export default defineConfig({
  base: './',
  plugins: [react()],
  resolve: { dedupe: ['react', 'react-dom'], alias: [{ find: /^loro-crdt$/, replacement: 'loro-crdt/browser' }] },
  optimizeDeps: {
    exclude: ['@kition/sdk', 'loro-crdt'],
    // The published kit editor reaches a CommonJS shim (through its UI library); excluded above, it
    // is not scanned, so the shim is named here to be converted for the browser.
    include: ['@kition/sdk > use-sync-external-store/shim'],
  },
  worker: { format: 'es' },
  // The development host (dev.html) and the kition-sdk and Tessera checkouts KITION_LINK=1 links.
  server: { fs: { allow: [fileURLToPath(new URL('..', import.meta.url)), fileURLToPath(new URL('../../kition-sdk', import.meta.url)), fileURLToPath(new URL('../../kition-tessera', import.meta.url))] } },
  build: { outDir: 'dist' },
})
