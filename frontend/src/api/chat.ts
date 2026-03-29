import api from './index'

export interface ChatConversation {
  id: string
  conversation_type: string
  peer_user_id: string
  peer_name: string
  peer_role: string
  last_message: string | null
  last_message_at: string | null
  unread_count: number
  updated_at: string
}

export interface ChatMessage {
  id: string
  conversation_id: string
  sender_id: string
  sender_name: string
  content: string
  message_type: string
  created_at: string
  is_self: boolean
}

export interface ChatSendMessageRequest {
  content: string
  message_type?: string
}

export interface ReadReceiptResponse {
  conversation_id: string
  read_at: string
}

export const chatApi = {
  listConversations() {
    return api.get<ChatConversation[]>('/chat/conversations')
  },

  listMessages(conversationId: string) {
    return api.get<ChatMessage[]>(`/chat/conversations/${conversationId}/messages`)
  },

  sendMessage(conversationId: string, request: ChatSendMessageRequest) {
    return api.post<ChatMessage>(`/chat/conversations/${conversationId}/messages`, request)
  },

  markRead(conversationId: string) {
    return api.post<ReadReceiptResponse>(`/chat/conversations/${conversationId}/read`)
  }
}
