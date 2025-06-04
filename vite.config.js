import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [
    react()
  ],
  // 防止 Vite 处理 Tauri 的 API 导入
  optimizeDeps: {
    exclude: ['@tauri-apps/api']
  },
  // 确保 Vite 正确处理 Tauri 的 API
  build: {
    target: ['es2021', 'chrome100', 'safari13'],
    minify: !process.env.TAURI_DEBUG ? 'esbuild' : false,
    sourcemap: !!process.env.TAURI_DEBUG,
    rollupOptions: {
      external: ['@tauri-apps/api'],
      output: {
        format: 'es'
      }
    }
  }
})
