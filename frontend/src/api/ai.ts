import api from './index'

export interface ChatRequest {
  message: string
  identity_id?: string
}

export interface ChatResponse {
  data: string
}

export interface AIIdentity {
  id: string
  name: string
  description: string
  prompt: string
  allowed_roles: string[]
  is_active: boolean
}

export interface CreateIdentityRequest {
  name: string
  description: string
  prompt: string
  allowed_roles: string[]
  is_active: boolean
}

export interface UpdateIdentityRequest {
  name?: string
  description?: string
  prompt?: string
  allowed_roles?: string[]
  is_active?: boolean
}

export interface AISettings {
  api_key: string
  api_base_url: string
  model: string
  default_prompt: string
  temperature: number
  max_tokens: number
}

export interface UpdateAISettingsRequest {
  api_key?: string
  api_base_url?: string
  model?: string
  default_prompt?: string
  temperature?: number
  max_tokens?: number
}

export interface AIContextData {
  classes: Array<{
    id: string
    name: string
    grade: number
    teacher_id: string | null
  }>
  groups: Array<{
    id: string
    name: string
    class_id: string
  }>
  departments: Array<{
    id: string
    name: string
  }>
}

// 数据查询相关接口
export interface DataQueryRequest {
  query_type: string
  id?: string
  format_as_markdown: boolean
}

export interface DataQueryResponse {
  data: string
  data_type: 'json' | 'markdown'
  user_permissions: string[]
}

// 增强版AI聊天接口
export interface ChatMessage {
  role: 'user' | 'assistant' | 'system'
  content: string
}

export interface EnhancedChatRequest {
  message: string
  conversation_history: ChatMessage[]
}

export interface EnhancedChatResponse {
  data: string
  query_executed: boolean
  query_type: string | null
}

export interface PageContextRequest {
  page: string
  path: string
  params?: Record<string, any>
  query?: Record<string, any>
}

export interface PageContextResponse {
  page: string
  data: Record<string, any>
}

export interface AssistantSuggestionRequest {
  page_context: Record<string, any>
  path?: string
  name?: string
}

export interface AssistantSuggestionResponse {
  suggestion: string
}

// AI 操作相关接口
export interface AIActionRequest {
  action_type: string
  params: Record<string, any>
  reason: string
  batch?: boolean
  items?: any[]
}

export interface NameCandidate {
  id: string
  name: string
  info: string
}

export interface BatchItemResult {
  success: boolean
  index: number
  data?: any
  error?: string
}

export interface BatchOperationResult {
  total: number
  success_count: number
  failure_count: number
  items: BatchItemResult[]
}

export interface AIActionResponse {
  success: boolean
  message: string
  data: any
  user_permissions: string[]
  need_confirmation: boolean
  candidates: NameCandidate[] | null
  batch_result?: BatchOperationResult
}

export interface AvailableAction {
  action_type: string
  name: string
  description: string
  required_params: string[]
  optional_params: string[]
}

export interface AvailableActionsResponse {
  available_actions: AvailableAction[]
  user_permissions: string[]
}

export const aiApi = {
  // 聊天
  chat(request: ChatRequest) {
    return api.post<ChatResponse>('/ai/chat', request)
  },

  // 列出 AI 身份
  listIdentities() {
    return api.get<AIIdentity[]>('/ai/identities')
  },

  // 创建 AI 身份
  createIdentity(request: CreateIdentityRequest) {
    return api.post<AIIdentity>('/ai/identities', request)
  },

  // 更新 AI 身份
  updateIdentity(id: string, request: UpdateIdentityRequest) {
    return api.put<AIIdentity>(`/ai/identities/${id}`, request)
  },

  // 删除 AI 身份
  deleteIdentity(id: string) {
    return api.delete(`/ai/identities/${id}`)
  },

  // 获取 AI 设置
  getSettings() {
    return api.get<AISettings>('/ai/settings')
  },

  // 更新 AI 设置
  updateSettings(request: UpdateAISettingsRequest) {
    return api.put<AISettings>('/ai/settings', request)
  },

  // 获取上下文数据
  getContextData() {
    return api.get<AIContextData>('/ai/context-data')
  },

  // 数据查询API - 供AI调用
  queryData(request: DataQueryRequest) {
    return api.post<DataQueryResponse>('/ai/query', request)
  },

  // 增强版AI聊天 - 支持自动数据查询（超时时间60秒，因为可能需要多次AI调用）
  enhancedChat(request: EnhancedChatRequest) {
    return api.post<EnhancedChatResponse>('/ai/enhanced-chat', request, {
      timeout: 60000
    })
  },

  // 执行AI操作
  executeAction(request: AIActionRequest) {
    return api.post<AIActionResponse>('/ai/actions', request)
  },

  // 获取用户可用的AI操作列表
  getAvailableActions() {
    return api.get<AvailableActionsResponse>('/ai/actions/available')
  },

  // 获取页面上下文
  getPageContext(request: PageContextRequest) {
    return api.post<PageContextResponse>('/ai/context', request)
  },

  // 获取页面顶部AI助手建议
  getAssistantSuggestion(request: AssistantSuggestionRequest) {
    return api.post<AssistantSuggestionResponse>('/ai/assistant/suggestion', request)
  }
}
