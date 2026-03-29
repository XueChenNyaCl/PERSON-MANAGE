<template>
  <div class="chat-view">
    <div class="chat-header">
      <div>
        <h2>在线对话</h2>
        <p>系统会自动为你加载可聊天对象，不支持手动新增会话。</p>
      </div>
      <el-badge :value="chatStore.totalUnreadCount" :hidden="chatStore.totalUnreadCount === 0">
        <el-button :loading="chatStore.loadingConversations" @click="reloadConversations">刷新会话</el-button>
      </el-badge>
    </div>

    <el-empty
      v-if="!chatStore.loadingConversations && chatStore.conversations.length === 0"
      description="当前暂无可用会话"
    >
      <template #default>
        <div class="chat-empty-tip">学生和教师会自动生成默认对话窗口，家长当前版本暂不开放。</div>
      </template>
    </el-empty>

    <div v-else class="chat-layout">
      <div class="conversation-panel">
        <div class="panel-title">会话列表</div>
        <el-scrollbar class="conversation-list">
          <div
            v-for="conversation in chatStore.conversations"
            :key="conversation.id"
            class="conversation-item"
            :class="{ active: conversation.id === chatStore.activeConversationId }"
            @click="selectConversation(conversation.id)"
          >
            <div class="conversation-main">
              <div class="conversation-name-row">
                <span class="conversation-name">{{ conversation.peer_name }}</span>
                <el-tag size="small" effect="plain">{{ formatRole(conversation.peer_role) }}</el-tag>
              </div>
              <div class="conversation-preview">{{ conversation.last_message || '暂无消息，快开始聊天吧' }}</div>
            </div>
            <div class="conversation-meta">
              <span class="conversation-time">{{ formatTime(conversation.last_message_at || conversation.updated_at) }}</span>
              <el-badge :value="conversation.unread_count" :hidden="conversation.unread_count === 0" />
            </div>
          </div>
        </el-scrollbar>
      </div>

      <div class="message-panel">
        <template v-if="activeConversation">
          <div class="message-panel-header">
            <div>
              <div class="peer-name">{{ activeConversation.peer_name }}</div>
              <div class="peer-role">{{ formatRole(activeConversation.peer_role) }}</div>
            </div>
          </div>

          <el-scrollbar ref="messageScrollbarRef" class="message-list">
            <div v-if="chatStore.loadingMessages" class="message-loading">
              <el-skeleton :rows="4" animated />
            </div>
            <div v-else-if="messages.length === 0" class="message-empty">
              <el-empty description="暂无聊天记录" />
            </div>
            <div v-else class="message-items">
              <div
                v-for="message in messages"
                :key="message.id"
                class="message-item"
                :class="{ self: message.is_self }"
              >
                <div class="message-bubble">
                  <div class="message-sender">{{ message.is_self ? '我' : message.sender_name }}</div>
                  <div class="message-content">{{ message.content }}</div>
                  <div class="message-time-inline">{{ formatDateTime(message.created_at) }}</div>
                </div>
              </div>
            </div>
          </el-scrollbar>

          <div class="message-composer">
            <el-input
              v-model="draft"
              type="textarea"
              :rows="3"
              resize="none"
              placeholder="请输入消息内容，按 Ctrl + Enter 发送"
              :disabled="chatStore.sending"
              @keydown="handleComposerKeydown"
            />
            <div class="composer-actions">
              <span class="composer-tip">仅支持系统自动生成的会话，不支持手动新增。</span>
              <el-button type="primary" :loading="chatStore.sending" :disabled="!draft.trim()" @click="submitMessage">
                发送
              </el-button>
            </div>
          </div>
        </template>

        <div v-else class="message-placeholder">
          <el-empty description="请选择一个会话开始聊天" />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from 'vue'
import { ElMessage } from 'element-plus'
import { useChatStore } from '../store/chat'

type ScrollbarInstance = {
  setScrollTop: (value: number) => void
  wrapRef?: HTMLElement
}

const chatStore = useChatStore()
const draft = ref('')
const messageScrollbarRef = ref<ScrollbarInstance | null>(null)

const roleLabelMap: Record<string, string> = {
  admin: '管理员',
  teacher: '教师',
  student: '学生',
  parent: '家长'
}

const activeConversation = computed(() => chatStore.activeConversation)
const messages = computed(() => {
  const conversationId = chatStore.activeConversationId
  return conversationId ? chatStore.getMessages(conversationId) : []
})

const formatRole = (role: string) => roleLabelMap[role] || role

const formatTime = (value: string) => {
  const date = new Date(value)
  const now = new Date()
  const sameDay = date.toDateString() === now.toDateString()
  if (sameDay) {
    return date.toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit' })
  }
  return date.toLocaleDateString('zh-CN', { month: '2-digit', day: '2-digit' })
}

const formatDateTime = (value: string) => {
  return new Date(value).toLocaleString('zh-CN')
}

const scrollMessagesToBottom = async () => {
  await nextTick()
  const wrap = messageScrollbarRef.value?.wrapRef
  if (wrap) {
    messageScrollbarRef.value?.setScrollTop(wrap.scrollHeight)
  }
}

const ensureMessagesLoaded = async (conversationId: string) => {
  await chatStore.loadMessages(conversationId)
  await scrollMessagesToBottom()
}

const selectConversation = async (conversationId: string) => {
  if (!conversationId) {
    return
  }

  chatStore.setActiveConversation(conversationId)
  draft.value = ''

  try {
    await ensureMessagesLoaded(conversationId)
  } catch (error: any) {
    ElMessage.error(error?.response?.data?.message || '加载聊天记录失败')
  }
}

const reloadConversations = async () => {
  try {
    await chatStore.refreshConversations()
    if (chatStore.activeConversationId) {
      await ensureMessagesLoaded(chatStore.activeConversationId)
    }
  } catch (error: any) {
    ElMessage.error(error?.response?.data?.message || '加载会话列表失败')
  }
}

const submitMessage = async () => {
  const conversationId = chatStore.activeConversationId
  const content = draft.value.trim()

  if (!conversationId || !content) {
    return
  }

  try {
    await chatStore.sendMessage(conversationId, content)
    draft.value = ''
    await scrollMessagesToBottom()
  } catch (error: any) {
    ElMessage.error(error?.response?.data?.message || '发送消息失败')
  }
}

const handleComposerKeydown = (event: KeyboardEvent) => {
  if (event.key === 'Enter' && event.ctrlKey) {
    event.preventDefault()
    void submitMessage()
  }
}

watch(
  () => messages.value.length,
  async () => {
    await scrollMessagesToBottom()
  }
)

onMounted(async () => {
  await reloadConversations()
})
</script>

<style scoped>
.chat-view {
  display: flex;
  flex-direction: column;
  gap: 16px;
  height: calc(100vh - 140px);
}

.chat-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 16px;
}

.chat-header h2 {
  margin: 0 0 6px;
}

.chat-header p {
  margin: 0;
  color: #909399;
}

.chat-empty-tip {
  color: #909399;
  font-size: 14px;
}

.chat-layout {
  display: grid;
  grid-template-columns: 320px 1fr;
  gap: 16px;
  min-height: 0;
  flex: 1;
}

.conversation-panel,
.message-panel {
  background: #fff;
  border-radius: 12px;
  padding: 16px;
  box-shadow: 0 8px 24px rgba(15, 23, 42, 0.06);
  min-height: 0;
}

.conversation-panel {
  display: flex;
  flex-direction: column;
}

.panel-title {
  font-size: 16px;
  font-weight: 600;
  margin-bottom: 12px;
}

.conversation-list {
  flex: 1;
}

.conversation-item {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  padding: 12px;
  border-radius: 10px;
  cursor: pointer;
  transition: all 0.2s ease;
  border: 1px solid transparent;
}

.conversation-item + .conversation-item {
  margin-top: 8px;
}

.conversation-item:hover,
.conversation-item.active {
  background: #f5f7ff;
  border-color: #c6d2ff;
}

.conversation-main {
  flex: 1;
  min-width: 0;
}

.conversation-name-row {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 6px;
}

.conversation-name {
  font-weight: 600;
}

.conversation-preview {
  color: #606266;
  font-size: 13px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.conversation-meta {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 8px;
  color: #909399;
  font-size: 12px;
}

.message-panel {
  display: flex;
  flex-direction: column;
}

.message-panel-header {
  padding-bottom: 12px;
  border-bottom: 1px solid #ebeef5;
  margin-bottom: 12px;
}

.peer-name {
  font-size: 18px;
  font-weight: 600;
}

.peer-role {
  margin-top: 4px;
  color: #909399;
  font-size: 13px;
}

.message-list {
  flex: 1;
  min-height: 0;
}

.message-items {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.message-item {
  display: flex;
}

.message-item.self {
  justify-content: flex-end;
}

.message-bubble {
  max-width: min(75%, 560px);
  background: #f5f7fa;
  border-radius: 12px;
  padding: 10px 12px;
}

.message-item.self .message-bubble {
  background: #dbe8ff;
}

.message-sender {
  font-size: 12px;
  color: #909399;
  margin-bottom: 4px;
}

.message-content {
  white-space: pre-wrap;
  word-break: break-word;
  line-height: 1.6;
}

.message-time-inline {
  margin-top: 6px;
  font-size: 12px;
  color: #909399;
  text-align: right;
}

.message-loading,
.message-empty,
.message-placeholder {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
}

.message-composer {
  margin-top: 12px;
  padding-top: 12px;
  border-top: 1px solid #ebeef5;
}

.composer-actions {
  margin-top: 10px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
}

.composer-tip {
  color: #909399;
  font-size: 12px;
}

@media (max-width: 900px) {
  .chat-view {
    height: auto;
  }

  .chat-layout {
    grid-template-columns: 1fr;
  }

  .conversation-panel,
  .message-panel {
    min-height: 320px;
  }

  .composer-actions {
    flex-direction: column;
    align-items: flex-start;
  }
}
</style>
