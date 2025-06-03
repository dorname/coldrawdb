import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import { tauri } from "@tauri-apps/api/vite";

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [
    react(),
    tauri()
  ],
})
