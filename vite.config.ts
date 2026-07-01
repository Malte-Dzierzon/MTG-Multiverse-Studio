import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

export default defineConfig({
  plugins: [react()],

  // Tauri expects a fixed port, so we don't use the dev server's default random behavior.
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      usePolling: true,
    },
  },

  // Make env vars accessible via import.meta.env
  envPrefix: ['TAURI_', 'VITE_'],
});
