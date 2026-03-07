import { ref } from 'vue'
import { useRoute } from 'vue-router'
import { aiApi, type PageContextResponse } from '../api/ai'

export interface PageContext {
  page: string
  path: string
  data: Record<string, any>
  timestamp: string
}

const resolvePageType = (path: string): string => {
  if (path.includes('/person')) return 'person'
  if (path.includes('/attendance')) return 'attendance'
  if (path.includes('/notice')) return 'notice'
  if (path.includes('/class')) return 'class'
  if (path.includes('/group')) return 'group'
  return 'dashboard'
}

const formatDateTime = (date: Date): string => date.toISOString()

export function usePageContext() {
  const route = useRoute()
  const pageContext = ref<PageContext | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  const getContext = async (): Promise<PageContext | null> => {
    if (loading.value) {
      return pageContext.value
    }

    loading.value = true
    error.value = null

    try {
      const page = resolvePageType(route.path)
      const response = await aiApi.getPageContext({
        page,
        path: route.path,
        params: route.params as Record<string, any>,
        query: route.query as Record<string, any>
      })

      const payload = response.data as PageContextResponse
      pageContext.value = {
        page,
        path: route.path,
        data: payload?.data ?? {},
        timestamp: formatDateTime(new Date())
      }

      return pageContext.value
    } catch (err: any) {
      error.value = err?.message || '获取页面上下文失败'
      return pageContext.value
    } finally {
      loading.value = false
    }
  }

  return {
    pageContext,
    loading,
    error,
    getContext
  }
}
