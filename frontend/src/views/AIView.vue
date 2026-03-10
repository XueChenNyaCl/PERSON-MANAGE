<template>
  <div class="ai-view">
    <div class="ai-header">
      <h2>AI 助手</h2>
      <p>与 AI 助手对话，获取您权限范围内的信息和分析</p>
    </div>
    
    <div class="ai-chat-container">
      <div class="ai-messages" ref="messagesContainer">
        <div
          v-for="(message, index) in messages"
          :key="index"
          class="ai-message"
          :class="message.role"
        >
          <div class="ai-message-avatar">
            <el-icon v-if="message.role === 'assistant'"><ChatLineRound /></el-icon>
            <el-icon v-else><User /></el-icon>
          </div>
          <div class="ai-message-content">
            <div v-if="message.queryExecuted" class="query-indicator">
              <el-tag size="small" type="success">
                <el-icon><DataLine /></el-icon>
                已自动查询数据
              </el-tag>
            </div>
            <div v-if="message.actionPending" class="action-indicator">
              <el-tag size="small" type="warning">
                <el-icon><Operation /></el-icon>
                待执行操作
              </el-tag>
              <el-button
                v-if="message.pendingAction"
                size="small"
                type="primary"
                @click="showActionDialog(message.pendingAction)"
              >
                查看详情
              </el-button>
            </div>
            <MarkdownRenderer v-if="message.isMarkdown" :content="message.content" />
            <template v-else>{{ message.content }}</template>
          </div>
        </div>
        
        <div v-if="loading" class="ai-message assistant">
          <div class="ai-message-avatar">
            <el-icon><ChatLineRound /></el-icon>
          </div>
          <div class="ai-message-content">
            <el-skeleton :rows="2" animated />
          </div>
        </div>
      </div>
      
      <div class="ai-input-container">
        <!-- 快捷查询按钮 -->
        <div class="quick-queries">
          <el-button 
            v-for="query in quickQueries" 
            :key="query.type"
            size="small"
            @click="executeQuickQuery(query)"
            :loading="loading && currentQueryType === query.type"
          >
            {{ query.label }}
          </el-button>
          <el-divider direction="vertical" v-if="availableActions.length > 0" />
          <el-button
            v-for="action in availableActions.slice(0, 3)"
            :key="action.action_type"
            size="small"
            type="primary"
            plain
            @click="sendMessageWithAction(action)"
            :loading="loading"
          >
            {{ action.name }}
          </el-button>
        </div>
        
        <div class="ai-input-wrapper">
          <el-input
            v-model="inputMessage"
            class="ai-input"
            placeholder="请输入您的问题，例如：'帮我统计一下每个部门有多少老师'..."
            :disabled="loading"
            @keyup.enter="sendMessage"
          />
          <el-button
            type="primary"
            class="ai-send-button"
            :disabled="loading || !inputMessage.trim()"
            @click="sendMessage"
          >
            发送
          </el-button>
        </div>
      </div>
    </div>

    <!-- AI操作执行对话框 -->
    <AIActionExecutor
      v-model="actionDialogVisible"
      :action-data="pendingActionData"
      @success="handleActionSuccess"
      @error="handleActionError"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, nextTick, watch } from 'vue'
import { ChatLineRound, User, DataLine, Operation } from '@element-plus/icons-vue'
import { ElMessage } from 'element-plus'
import { aiApi, type ChatMessage, type AIActionRequest, type AIActionResponse } from '../api/ai'
import MarkdownRenderer from '../components/MarkdownRenderer.vue'
import AIActionExecutor from '../components/AIActionExecutor.vue'
import { useAIStore } from '../store/ai'
import { useMobile } from '../composables/useMobile'

interface Message {
  role: 'user' | 'assistant' | 'system'
  content: string
  isMarkdown?: boolean
  queryExecuted?: boolean
  actionPending?: boolean
  pendingAction?: AIActionRequest
}

interface QuickQuery {
  type: string
  label: string
  query_type: string
}

const defaultWelcomeMessage: Message = {
  role: 'assistant',
  content: '你好！我是学校管理系统的AI助手，可以帮你查询和分析学校数据，并根据你的权限执行相应操作。\n\n**我可以帮你：**\n- 查询班级列表和详情\n- 查询小组信息\n- 查询部门信息\n- 统计分析数据（如：每个部门有多少老师）\n- 获取学校数据概览\n- 根据你的权限执行操作（如创建公告、管理小组等）\n\n你可以直接输入自然语言问题，例如：\n- "帮我统计一下每个部门有多少老师"\n- "查看所有班级信息"\n- "创建一个期中考试通知公告"\n- "给第一小组增加10分"\n\n或者直接点击下方快捷按钮。',
  isMarkdown: true
}

const aiStore = useAIStore()

const initialMessages = aiStore.chatMessages.length > 0
  ? aiStore.chatMessages
  : [defaultWelcomeMessage]

const messages = ref<Message[]>(initialMessages)

const inputMessage = ref(aiStore.chatInput || '')
const loading = ref(false)
const currentQueryType = ref('')
const messagesContainer = ref<HTMLElement | null>(null)

const quickQueries: QuickQuery[] = [
  { type: 'class_list', label: '📚 班级列表', query_type: 'class_list' },
  { type: 'group_list', label: '👥 小组列表', query_type: 'group_list' },
  { type: 'department_list', label: '🏢 部门列表', query_type: 'department_list' },
  { type: 'overview', label: '📊 数据概览', query_type: 'overview' }
]

// 可用的AI操作
const availableActions = ref<Array<{action_type: string, name: string, description: string}>>([])
const userPermissions = ref<string[]>([])

// 操作执行对话框状态
const actionDialogVisible = ref(false)
const pendingActionData = ref<AIActionRequest | undefined>(undefined)

// 加载用户可用操作
const loadAvailableActions = async () => {
  try {
    const response = await aiApi.getAvailableActions()
    availableActions.value = response.data.available_actions
    userPermissions.value = response.data.user_permissions
  } catch (error) {
    console.error('加载可用操作失败:', error)
  }
}

// 移动端适配
const { isMobile, loadMobileStyle } = useMobile()

// 页面加载时获取可用操作
loadAvailableActions()

// 动态加载移动端样式
if (isMobile.value) {
  loadMobileStyle('ai-assistant-mobile')
}

watch(messages, (newMessages) => {
  aiStore.setChatMessages(newMessages)
}, { deep: true })

watch(inputMessage, (newInput) => {
  aiStore.setChatInput(newInput)
})

// 显示操作对话框
const showActionDialog = (action: AIActionRequest) => {
  pendingActionData.value = action
  actionDialogVisible.value = true
}

// 处理操作成功
const handleActionSuccess = (result: AIActionResponse) => {
  messages.value.push({
    role: 'assistant',
    content: `✅ 操作执行成功！\n\n${result.message}`,
    isMarkdown: true
  })
  scrollToBottom()
}

// 处理操作失败
const handleActionError = (error: any) => {
  messages.value.push({
    role: 'assistant',
    content: `❌ 操作执行失败！\n\n${error.message || '未知错误'}`,
    isMarkdown: true
  })
  scrollToBottom()
}

const scrollToBottom = async () => {
  await nextTick()
  if (messagesContainer.value) {
    messagesContainer.value.scrollTop = messagesContainer.value.scrollHeight
  }
}

const pushAssistantSegments = (rawContent: string, queryExecuted = false) => {
  const parts = rawContent
    .split('[[AI_SEGMENT]]')
    .map(part => part.trim())
    .filter(Boolean)

  const segments = parts.length > 0 ? parts : [rawContent]

  segments.forEach((segment, index) => {
    messages.value.push({
      role: 'assistant',
      content: segment,
      isMarkdown: true,
      queryExecuted: queryExecuted && index === 0
    })
  })
}

// 转换消息格式用于API调用
const getConversationHistory = (): ChatMessage[] => {
  return messages.value
    .filter(m => m.role !== 'system')
    .map(m => ({
      role: m.role,
      content: m.content
    }))
}

const sendMessage = async () => {
  if (!inputMessage.value.trim() || loading.value) return
  
  const userMessage = inputMessage.value.trim()
  messages.value.push({
    role: 'user',
    content: userMessage
  })
  
  inputMessage.value = ''
  loading.value = true
  await scrollToBottom()
  
  try {
    // 使用增强版聊天API
    const response = await aiApi.enhancedChat({
      message: userMessage,
      conversation_history: getConversationHistory()
    })

    pushAssistantSegments(response.data.data, response.data.query_executed)
  } catch (error: any) {
    console.error('Chat error:', error)
    const errorMsg = error.response?.data?.message || '与 AI 对话失败，请稍后重试'
    ElMessage.error(errorMsg)
    messages.value.push({
      role: 'assistant',
      content: `抱歉，${errorMsg}`
    })
  } finally {
    loading.value = false
    await scrollToBottom()
  }
}

const executeQuickQuery = async (query: QuickQuery) => {
  if (loading.value) return

  loading.value = true
  currentQueryType.value = query.type

  const userMessage = query.label
  messages.value.push({
    role: 'user',
    content: userMessage
  })

  await scrollToBottom()

  try {
    const response = await aiApi.enhancedChat({
      message: userMessage,
      conversation_history: getConversationHistory()
    })

    pushAssistantSegments(response.data.data, response.data.query_executed)
  } catch (error: any) {
    console.error('Query error:', error)
    const errorMsg = error.response?.data?.message || '查询失败，请稍后重试'
    ElMessage.error(errorMsg)
    messages.value.push({
      role: 'assistant',
      content: `抱歉，${errorMsg}`
    })
  } finally {
    loading.value = false
    currentQueryType.value = ''
    await scrollToBottom()
  }
}

// 发送带操作意图的消息
const sendMessageWithAction = (action: {action_type: string, name: string, description: string}) => {
  inputMessage.value = `帮我${action.name}，${action.description}`
  sendMessage()
}
</script>

<style scoped src="@/styles/ai-view.css"></style>
