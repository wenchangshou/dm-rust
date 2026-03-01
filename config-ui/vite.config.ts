import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

// 后端 API 地址，可通过环境变量 VITE_API_BASE 配置
// 例如: VITE_API_BASE=http://192.168.1.100:18080 npm run dev
const apiBase = process.env.VITE_API_BASE || 'http://localhost:18080'

export default defineConfig({
  plugins: [vue()],
  base: '/config/',
  build: {
    outDir: '../dist-config',
    emptyOutDir: true,
  },
  server: {
    port: 5173,
    proxy: {
      '/lspcapi': {
        target: apiBase,
        changeOrigin: true,
      }
    }
  }
})
