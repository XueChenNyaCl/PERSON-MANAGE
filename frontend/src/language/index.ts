import { createI18n } from 'vue-i18n'

// 导入所有语言文件
const messages = Object.fromEntries(
  Object.entries((import.meta as any).glob('./*.json', { eager: true }))
    .map(([key, value]) => {
      const fileName = key.replace('./', '').replace('.json', '')
      return [fileName, (value as any).default]
    })
)

// 创建 i18n 实例
export const i18n = createI18n({
  legacy: false,
  locale: 'zh-CN',
  fallbackLocale: 'zh-CN',
  messages
})

// 导出常用的辅助函数
// export const t = i18n.global.t
export const setLocale = (locale: string) => {
  i18n.global.locale.value = locale
}
export const getLocale = () => i18n.global.locale.value
export const getLocaleRef = () => i18n.global.locale
export const getAvailableLocales = () => Object.keys(messages)
export const isSupportedLocale = (locale: string) => locale in messages