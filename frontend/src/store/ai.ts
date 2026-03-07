import { defineStore } from 'pinia'
import { ref } from 'vue'

const DISMISSED_STORAGE_KEY = 'ai_assistant_dismissed_paths'
const CHAT_CONTEXT_STORAGE_KEY = 'ai_chat_context_v1'

export interface AIChatCachedMessage {
  role: 'user' | 'assistant' | 'system'
  content: string
  isMarkdown?: boolean
  queryExecuted?: boolean
}

interface AIChatContext {
  messages: AIChatCachedMessage[]
  input: string
}

export const useAIStore = defineStore('ai', () => {
  const dismissedPaths = ref<string[]>([])
  const chatMessages = ref<AIChatCachedMessage[]>([])
  const chatInput = ref('')

  const dismissedRaw = localStorage.getItem(DISMISSED_STORAGE_KEY)
  if (dismissedRaw) {
    try {
      dismissedPaths.value = JSON.parse(dismissedRaw)
    } catch {
      dismissedPaths.value = []
    }
  }

  const chatContextRaw = localStorage.getItem(CHAT_CONTEXT_STORAGE_KEY)
  if (chatContextRaw) {
    try {
      const parsed = JSON.parse(chatContextRaw) as AIChatContext
      chatMessages.value = Array.isArray(parsed.messages) ? parsed.messages : []
      chatInput.value = typeof parsed.input === 'string' ? parsed.input : ''
    } catch {
      chatMessages.value = []
      chatInput.value = ''
    }
  }

  const persistDismissed = () => {
    localStorage.setItem(DISMISSED_STORAGE_KEY, JSON.stringify(dismissedPaths.value))
  }

  const persistChatContext = () => {
    localStorage.setItem(CHAT_CONTEXT_STORAGE_KEY, JSON.stringify({
      messages: chatMessages.value,
      input: chatInput.value
    }))
  }

  const isDismissed = (path: string) => dismissedPaths.value.includes(path)

  const dismiss = (path: string) => {
    if (!dismissedPaths.value.includes(path)) {
      dismissedPaths.value.push(path)
      persistDismissed()
    }
  }

  const clearDismissed = (path: string) => {
    dismissedPaths.value = dismissedPaths.value.filter(p => p !== path)
    persistDismissed()
  }

  const setChatMessages = (messages: AIChatCachedMessage[]) => {
    chatMessages.value = messages
    persistChatContext()
  }

  const setChatInput = (input: string) => {
    chatInput.value = input
    persistChatContext()
  }

  const clearChatContext = () => {
    chatMessages.value = []
    chatInput.value = ''
    persistChatContext()
  }

  return {
    dismissedPaths,
    chatMessages,
    chatInput,
    isDismissed,
    dismiss,
    clearDismissed,
    setChatMessages,
    setChatInput,
    clearChatContext
  }
})
