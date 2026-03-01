import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { resolve } from 'path'

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      '@': resolve(__dirname, 'src'),
      '@src': resolve(__dirname, 'src'),
      '@api': resolve(__dirname, 'src/api'),
      '@assets': resolve(__dirname, 'src/assets'),
      '@components': resolve(__dirname, 'src/components'),
      '@composables': resolve(__dirname, 'src/composables'),
      '@data': resolve(__dirname, 'src/data'),
      '@language': resolve(__dirname, 'src/language'),
      '@router': resolve(__dirname, 'src/router'),
      '@stores': resolve(__dirname, 'src/stores'),
      '@styles': resolve(__dirname, 'src/styles'),
      '@themes': resolve(__dirname, 'src/themes'),
      '@type': resolve(__dirname, 'src/types'),
      '@utils': resolve(__dirname, 'src/utils'),
      '@views': resolve(__dirname, 'src/views')
    }
  },
  server: {
    port: 3001,
    proxy: {
      '/api': {
        target: 'http://localhost:3000',
        changeOrigin: true
      }
    }
  }
})
