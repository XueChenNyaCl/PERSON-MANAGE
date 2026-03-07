import { defineStore } from 'pinia'
import { ref } from 'vue'

const STORAGE_KEY = 'ai_assistant_dismissed_paths'

export const useAIStore = defineStore('ai', () => {
  const dismissedPaths = ref<string[]>([])

  const raw = localStorage.getItem(STORAGE_KEY)
  if (raw) {
    try {
      dismissedPaths.value = JSON.parse(raw)
    } catch {
      dismissedPaths.value = []
    }
  }

  const persist = () => {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(dismissedPaths.value))
  }

  const isDismissed = (path: string) => dismissedPaths.value.includes(path)

  const dismiss = (path: string) => {
    if (!dismissedPaths.value.includes(path)) {
      dismissedPaths.value.push(path)
      persist()
    }
  }

  const clearDismissed = (path: string) => {
    dismissedPaths.value = dismissedPaths.value.filter(p => p !== path)
    persist()
  }

  return {
    dismissedPaths,
    isDismissed,
    dismiss,
    clearDismissed
  }
})
