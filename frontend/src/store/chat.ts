import { computed, ref } from 'vue'
import { defineStore } from 'pinia'
import { chatApi, type ChatConversation, type ChatMessage } from '../api/chat'

const CHAT_ACTIVE_KEY = 'chat_active_conversation_id'

export const useChatStore = defineStore('chat', () => {
  const conversations = ref<ChatConversation[]>([])
  const messageMap = ref<Record<string, ChatMessage[]>>({})
  const activeConversationId = ref(localStorage.getItem(CHAT_ACTIVE_KEY) || '')
  const loadingConversations = ref(false)
  const loadingMessages = ref(false)
  const sending = ref(false)

  const activeConversation = computed(() => {
    return conversations.value.find(item => item.id === activeConversationId.value) || null
  })

  const totalUnreadCount = computed(() => {
    return conversations.value.reduce((sum, item) => sum + item.unread_count, 0)
  })

  const getMessages = (conversationId: string) => messageMap.value[conversationId] || []

  const setActiveConversation = (conversationId: string) => {
    activeConversationId.value = conversationId
    if (conversationId) {
      localStorage.setItem(CHAT_ACTIVE_KEY, conversationId)
    } else {
      localStorage.removeItem(CHAT_ACTIVE_KEY)
    }
  }

  const upsertConversation = (conversation: ChatConversation) => {
    const next = [...conversations.value]
    const index = next.findIndex(item => item.id === conversation.id)

    if (index >= 0) {
      next[index] = conversation
    } else {
      next.push(conversation)
    }

    next.sort((a, b) => {
      const aTime = new Date(a.last_message_at || a.updated_at).getTime()
      const bTime = new Date(b.last_message_at || b.updated_at).getTime()
      return bTime - aTime
    })

    conversations.value = next
  }

  const refreshConversations = async () => {
    loadingConversations.value = true
    try {
      const { data } = await chatApi.listConversations()
      conversations.value = data

      if (!data.length) {
        setActiveConversation('')
        return data
      }

      const currentExists = data.some(item => item.id === activeConversationId.value)
      if (!currentExists) {
        setActiveConversation(data[0].id)
      }

      return data
    } finally {
      loadingConversations.value = false
    }
  }

  const loadMessages = async (conversationId: string, options?: { markRead?: boolean }) => {
    loadingMessages.value = true
    try {
      const { data } = await chatApi.listMessages(conversationId)
      messageMap.value = {
        ...messageMap.value,
        [conversationId]: data
      }

      if (options?.markRead !== false) {
        await markConversationRead(conversationId)
      }

      return data
    } finally {
      loadingMessages.value = false
    }
  }

  const markConversationRead = async (conversationId: string) => {
    await chatApi.markRead(conversationId)
    conversations.value = conversations.value.map(item => {
      if (item.id !== conversationId) {
        return item
      }
      return {
        ...item,
        unread_count: 0
      }
    })
  }

  const sendMessage = async (conversationId: string, content: string) => {
    sending.value = true
    try {
      const { data } = await chatApi.sendMessage(conversationId, { content })
      const existing = messageMap.value[conversationId] || []
      messageMap.value = {
        ...messageMap.value,
        [conversationId]: [...existing, data]
      }

      const target = conversations.value.find(item => item.id === conversationId)
      if (target) {
        upsertConversation({
          ...target,
          last_message: data.content,
          last_message_at: data.created_at,
          updated_at: data.created_at,
          unread_count: 0
        })
      }

      return data
    } finally {
      sending.value = false
    }
  }

  const clear = () => {
    conversations.value = []
    messageMap.value = {}
    setActiveConversation('')
  }

  return {
    conversations,
    messageMap,
    activeConversationId,
    activeConversation,
    totalUnreadCount,
    loadingConversations,
    loadingMessages,
    sending,
    getMessages,
    setActiveConversation,
    refreshConversations,
    loadMessages,
    markConversationRead,
    sendMessage,
    upsertConversation,
    clear
  }
})
