import { createApp } from 'vue'
import { createPinia } from 'pinia'
import ElementPlus from 'element-plus'
import 'element-plus/dist/index.css'
import '@/styles/main.css'
import App from './App.vue'
import router from './router'
import { i18n } from './language'

const app = createApp(App)

app.use(createPinia())
app.use(router)
app.use(ElementPlus)
app.use(i18n)

// 动态加载移动端基础样式
const loadMobileStyles = async () => {
  const isMobile = window.innerWidth <= 768
  if (isMobile) {
    try {
      await import('@/styles/mobile/mobile-base.css')
    } catch (error) {
      console.warn('[main] 加载移动端基础样式失败:', error)
    }
  }
}

// 监听窗口大小变化，动态加载/卸载移动端样式
let mobileStylesLoaded = false
const handleResize = () => {
  const isMobile = window.innerWidth <= 768
  if (isMobile && !mobileStylesLoaded) {
    loadMobileStyles()
    mobileStylesLoaded = true
  }
}

// 初始加载
loadMobileStyles()
window.addEventListener('resize', handleResize)

app.mount('#app')
