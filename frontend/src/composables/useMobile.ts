import { ref, computed, onMounted, onUnmounted } from 'vue'

/**
 * 移动端检测和适配 composable
 * 用于检测设备类型、屏幕尺寸，并动态加载移动端样式
 */

// 移动端断点
const MOBILE_BREAKPOINT = 768
const TABLET_BREAKPOINT = 1024

// 响应式状态
const windowWidth = ref(typeof window !== 'undefined' ? window.innerWidth : 1024)
const windowHeight = ref(typeof window !== 'undefined' ? window.innerHeight : 768)
const isMobile = computed(() => windowWidth.value <= MOBILE_BREAKPOINT)
const isTablet = computed(() => windowWidth.value > MOBILE_BREAKPOINT && windowWidth.value <= TABLET_BREAKPOINT)
const isDesktop = computed(() => windowWidth.value > TABLET_BREAKPOINT)

// 已加载的移动端样式集合
const loadedMobileStyles = new Set<string>()

/**
 * 动态加载移动端样式文件
 * @param styleName 样式文件名（不含.css后缀）
 */
export async function loadMobileStyle(styleName: string): Promise<void> {
  // 只在移动端加载
  if (!isMobile.value) return

  // 避免重复加载
  if (loadedMobileStyles.has(styleName)) return

  try {
    // 动态导入样式
    await import(`@/styles/mobile/${styleName}.css`)
    loadedMobileStyles.add(styleName)
  } catch (error) {
    console.warn(`[useMobile] 加载移动端样式失败: ${styleName}`, error)
  }
}

/**
 * 移除移动端样式
 * @param styleName 样式文件名（不含.css后缀）
 */
export function unloadMobileStyle(styleName: string): void {
  // 查找并移除样式标签
  const styleId = `mobile-style-${styleName}`
  const existingStyle = document.getElementById(styleId)
  if (existingStyle) {
    existingStyle.remove()
    loadedMobileStyles.delete(styleName)
  }
}

/**
 * 主要的移动端适配 composable
 */
export function useMobile() {
  // 更新窗口尺寸
  const updateWindowSize = () => {
    windowWidth.value = window.innerWidth
    windowHeight.value = window.innerHeight
  }

  // 监听窗口变化
  onMounted(() => {
    window.addEventListener('resize', updateWindowSize)
    // 同时监听方向变化
    window.addEventListener('orientationchange', updateWindowSize)
  })

  onUnmounted(() => {
    window.removeEventListener('resize', updateWindowSize)
    window.removeEventListener('orientationchange', updateWindowSize)
  })

  return {
    // 状态
    windowWidth,
    windowHeight,
    isMobile,
    isTablet,
    isDesktop,

    // 方法
    loadMobileStyle,
    unloadMobileStyle
  }
}

/**
 * 简化的移动端检测（用于非组件环境）
 */
export function checkIsMobile(): boolean {
  if (typeof window === 'undefined') return false
  return window.innerWidth <= MOBILE_BREAKPOINT
}

/**
 * 获取视口信息
 */
export function getViewportInfo() {
  if (typeof window === 'undefined') {
    return {
      width: 1024,
      height: 768,
      isMobile: false,
      isTablet: false,
      isDesktop: true
    }
  }

  const width = window.innerWidth
  return {
    width,
    height: window.innerHeight,
    isMobile: width <= MOBILE_BREAKPOINT,
    isTablet: width > MOBILE_BREAKPOINT && width <= TABLET_BREAKPOINT,
    isDesktop: width > TABLET_BREAKPOINT
  }
}

export default useMobile
