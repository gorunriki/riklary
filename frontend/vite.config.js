import { defineConfig } from 'vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'

// https://vite.dev/config/
export default defineConfig({
  plugins: [svelte()],
  server: {
    port: 5173,
    allowedHosts: true,
    proxy: {
      // Mengarahkan request API ke backend Rust
      '/api': {
        target: 'http://127.0.0.1:8080',
        changeOrigin: true,
      },
      // Mengarahkan request asset buku statis ke backend Rust
      '/static': {
        target: 'http://127.0.0.1:8080',
        changeOrigin: true,
      }
    }
  }
})
