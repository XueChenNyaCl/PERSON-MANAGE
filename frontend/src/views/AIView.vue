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
  </div>
</template>

<script setup lang="ts">
import { ref, nextTick } from 'vue'
import { ChatLineRound, User, DataLine } from '@element-plus/icons-vue'
import { ElMessage } from 'element-plus'
import { aiApi, type ChatMessage } from '../api/ai'
import MarkdownRenderer from '../components/MarkdownRenderer.vue'

interface Message {
  role: 'user' | 'assistant' | 'system'
  content: string
  isMarkdown?: boolean
  queryExecuted?: boolean
}

interface QuickQuery {
  type: string
  label: string
  query_type: string
}

const messages = ref<Message[]>([
  {
    role: 'assistant',
    content: '你好！我是学校管理系统的AI助手，可以帮你查询和分析学校数据，并根据你的权限执行相应操作。\n\n**我可以帮你：**\n- 查询班级列表和详情\n- 查询小组信息\n- 查询部门信息\n- 统计分析数据（如：每个部门有多少老师）\n- 获取学校数据概览\n- 根据你的权限执行操作（如创建公告、管理小组等）\n\n你可以直接输入自然语言问题，例如：\n- "帮我统计一下每个部门有多少老师"\n- "查看所有班级信息"\n- "创建一个期中考试通知公告"\n- "给第一小组增加10分"\n\n或者直接点击下方快捷按钮。',
    isMarkdown: true
  }
])

const inputMessage = ref('')
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

// 页面加载时获取可用操作
loadAvailableActions()

const scrollToBottom = async () => {
  await nextTick()
  if (messagesContainer.value) {
    messagesContainer.value.scrollTop = messagesContainer.value.scrollHeight
  }
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
    
    messages.value.push({
      role: 'assistant',
      content: response.data.data,
      isMarkdown: response.data.query_executed, // 如果执行了查询，使用Markdown渲染
      queryExecuted: response.data.query_executed
    })
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

    messages.value.push({
      role: 'assistant',
      content: response.data.data,
      isMarkdown: true,
      queryExecuted: response.data.query_executed
    })
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

<style scoped>
@import '../styles/ai-view.css';

.quick-queries {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  padding: 8px 16px;
  background-color: #f5f7fa;
  border-bottom: 1px solid #e4e7ed;
}

.quick-queries .el-button {
  font-size: 12px;
}

.query-indicator {
  margin-bottom: 8px;
}

.query-indicator .el-tag {
  display: inline-flex;
  align-items: center;
  gap: 4px;
}
</style>
