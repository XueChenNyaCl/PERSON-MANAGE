<template>
  <div v-if="visible" class="ai-action-executor">
    <el-dialog
      v-model="visible"
      :title="dialogTitle"
      width="500px"
      :close-on-click-modal="false"
      @close="handleClose"
    >
      <!-- 操作确认内容 -->
      <div v-if="actionData && !needConfirmation" class="action-content">
        <div class="action-info">
          <p><strong>操作类型：</strong>{{ actionName }}</p>
          <p v-if="actionData.reason"><strong>操作原因：</strong>{{ actionData.reason }}</p>
        </div>
        
        <div class="params-preview">
          <h4>操作参数：</h4>
          <pre>{{ JSON.stringify(actionData.params, null, 2) }}</pre>
        </div>

        <!-- 批量操作显示 -->
        <div v-if="isBatch && actionData.items" class="batch-info">
          <el-alert
            :title="`将执行 ${actionData.items.length} 条记录`"
            type="info"
            :closable="false"
          />
        </div>
      </div>

      <!-- 需要用户确认（重名情况） -->
      <div v-if="needConfirmation && candidates.length > 0" class="confirmation-content">
        <el-alert
          title="找到多个匹配项，请选择具体的一项"
          type="warning"
          :closable="false"
          show-icon
        />
        <div class="candidates-list">
          <el-radio-group v-model="selectedCandidate">
            <el-radio
              v-for="candidate in candidates"
              :key="candidate.id"
              :label="candidate.id"
            >
              {{ candidate.name }} - {{ candidate.info }}
            </el-radio>
          </el-radio-group>
        </div>
      </div>

      <!-- 执行结果 -->
      <div v-if="executionResult" class="execution-result">
        <el-alert
          :title="executionResult.success ? '执行成功' : '执行失败'"
          :type="executionResult.success ? 'success' : 'error'"
          :closable="false"
          show-icon
        />
        <div v-if="executionResult.data" class="result-data">
          <p>{{ executionResult.message }}</p>
          <!-- 批量操作结果显示 -->
          <div v-if="executionResult.data.items" class="batch-results">
            <el-collapse>
              <el-collapse-item :title="`查看详细结果 (${executionResult.data.success_count}/${executionResult.data.total})`">
                <div
                  v-for="(item, index) in executionResult.data.items"
                  :key="index"
                  class="batch-item"
                  :class="{ success: item.success, error: !item.success }"
                >
                  <span class="item-index">#{{ item.index + 1 }}</span>
                  <el-icon v-if="item.success" class="success-icon"><Check /></el-icon>
                  <el-icon v-else class="error-icon"><Close /></el-icon>
                  <span v-if="item.error" class="error-message">{{ item.error }}</span>
                </div>
              </el-collapse-item>
            </el-collapse>
          </div>
        </div>
      </div>

      <template #footer>
        <span class="dialog-footer">
          <el-button @click="handleClose">取消</el-button>
          <el-button
            v-if="needConfirmation && !executionResult"
            type="primary"
            :disabled="!selectedCandidate"
            @click="confirmSelection"
          >
            确认选择
          </el-button>
          <el-button
            v-else-if="!executionResult"
            type="primary"
            :loading="executing"
            @click="executeAction"
          >
            执行操作
          </el-button>
          <el-button
            v-else
            type="primary"
            @click="handleClose"
          >
            完成
          </el-button>
        </span>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { ElMessage } from 'element-plus'
import { Check, Close } from '@element-plus/icons-vue'
import { aiApi, type AIActionRequest, type NameCandidate, type AIActionResponse } from '../api/ai'

const props = defineProps<{
  modelValue: boolean
  actionData?: AIActionRequest
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void
  (e: 'success', result: AIActionResponse): void
  (e: 'error', error: any): void
}>()

const visible = computed({
  get: () => props.modelValue,
  set: (value) => emit('update:modelValue', value)
})

const executing = ref(false)
const needConfirmation = ref(false)
const candidates = ref<NameCandidate[]>([])
const selectedCandidate = ref('')
const executionResult = ref<AIActionResponse | null>(null)

const actionName = computed(() => {
  const nameMap: Record<string, string> = {
    'create_notice': '创建公告',
    'create_group': '创建小组',
    'update_group_score': '更新小组积分',
    'add_group_member': '添加小组成员',
    'remove_group_member': '移除小组成员',
    'create_attendance': '创建考勤记录',
    'create_attendances_batch': '批量创建考勤记录',
    'create_score': '添加个人积分',
    'create_scores_batch': '批量添加个人积分',
  }
  return nameMap[props.actionData?.action_type || ''] || props.actionData?.action_type || '未知操作'
})

const isBatch = computed(() => {
  return props.actionData?.action_type?.includes('_batch') || props.actionData?.batch
})

const dialogTitle = computed(() => {
  if (executionResult.value) {
    return executionResult.value.success ? '执行成功' : '执行失败'
  }
  if (needConfirmation.value) {
    return '请选择'
  }
  return `确认执行：${actionName.value}`
})

const executeAction = async () => {
  if (!props.actionData) return
  
  executing.value = true
  try {
    const response = await aiApi.executeAction(props.actionData)
    
    if (response.data.need_confirmation) {
      // 需要用户确认（重名情况）
      needConfirmation.value = true
      candidates.value = response.data.candidates || []
      selectedCandidate.value = ''
    } else {
      // 执行完成
      executionResult.value = response.data
      if (response.data.success) {
        emit('success', response.data)
        ElMessage.success(response.data.message)
      } else {
        emit('error', new Error(response.data.message))
        ElMessage.error(response.data.message)
      }
    }
  } catch (error: any) {
    emit('error', error)
    ElMessage.error(error.message || '执行失败')
  } finally {
    executing.value = false
  }
}

const confirmSelection = async () => {
  if (!props.actionData || !selectedCandidate.value) return
  
  // 更新参数中的ID为选中的ID
  const updatedAction = {
    ...props.actionData,
    params: {
      ...props.actionData.params,
      person_id: selectedCandidate.value
    }
  }
  
  executing.value = true
  try {
    const response = await aiApi.executeAction(updatedAction)
    executionResult.value = response.data
    needConfirmation.value = false
    
    if (response.data.success) {
      emit('success', response.data)
      ElMessage.success(response.data.message)
    } else {
      emit('error', new Error(response.data.message))
      ElMessage.error(response.data.message)
    }
  } catch (error: any) {
    emit('error', error)
    ElMessage.error(error.message || '执行失败')
  } finally {
    executing.value = false
  }
}

const handleClose = () => {
  visible.value = false
  // 重置状态
  executing.value = false
  needConfirmation.value = false
  candidates.value = []
  selectedCandidate.value = ''
  executionResult.value = null
}

// 暴露方法给父组件
defineExpose({
  executeAction,
  handleClose
})
</script>

<style scoped>
@import '../styles/ai-action-executor.css';
</style>
