<template>
  <div class="ai-settings-view">
    <div v-if="!hasPermission" class="no-permission">
      <h2>权限不足</h2>
      <p>您没有访问 AI 设置页面的权限。</p>
    </div>
    <div v-else class="ai-settings-container">
      <h2>AI 设置</h2>
      
      <el-form :model="settings" label-width="150px" class="settings-form">
        <el-form-item label="API 密钥">
          <el-input 
            v-model="settings.api_key" 
            type="password"
            show-password
            placeholder="请输入 API 密钥"
          />
        </el-form-item>
        
        <el-form-item label="API 基础 URL">
          <el-input 
            v-model="settings.api_base_url" 
            placeholder="https://api.openai.com/v1"
          />
        </el-form-item>
        
        <el-form-item label="模型">
          <el-input 
            v-model="settings.model" 
            placeholder="gpt-3.5-turbo"
          />
        </el-form-item>
        
        <el-form-item label="默认提示词">
          <el-input 
            v-model="settings.default_prompt" 
            type="textarea"
            :rows="4"
            placeholder="You are an AI assistant for a school management system."
          />
        </el-form-item>
        
        <el-form-item label="温度">
          <el-slider 
            v-model="temperatureValue" 
            :min="0" 
            :max="2" 
            :step="0.1"
            :format-tooltip="formatTemperature"
          />
        </el-form-item>
        
        <el-form-item label="最大 Token 数">
          <el-input-number 
            v-model="settings.max_tokens" 
            :min="100" 
            :max="8000" 
            :step="100"
          />
        </el-form-item>
        
        <el-form-item>
          <el-button type="primary" @click="saveSettings" :loading="saving">
            保存设置
          </el-button>
        </el-form-item>
      </el-form>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted, computed } from 'vue'
import { ElMessage } from 'element-plus'
import { aiApi, type AISettings } from '../api/ai'
import { useAuthStore } from '../store/auth'

const authStore = useAuthStore()

// 检查是否有 AI 设置权限
const hasPermission = computed(() => {
  return authStore.hasPermission('ai.settings')
})

const settings = reactive<AISettings>({
  api_key: '',
  api_base_url: 'https://api.openai.com/v1',
  model: 'gpt-3.5-turbo',
  default_prompt: 'You are an AI assistant for a school management system.',
  temperature: 0.7,
  max_tokens: 1000
})

const temperatureValue = ref(0.7)
const saving = ref(false)

const formatTemperature = (value: number) => {
  return value.toFixed(1)
}

const loadSettings = async () => {
  // 如果没有权限，不加载设置
  if (!hasPermission.value) {
    return
  }
  
  try {
    const response = await aiApi.getSettings()
    if (response.data) {
      Object.assign(settings, response.data)
      temperatureValue.value = settings.temperature
    }
  } catch (error: any) {
    if (error.response?.status !== 404) {
      ElMessage.error('加载设置失败：' + (error.message || '未知错误'))
    }
  }
}

const saveSettings = async () => {
  saving.value = true
  try {
    settings.temperature = temperatureValue.value
    const response = await aiApi.updateSettings({
      api_key: settings.api_key,
      api_base_url: settings.api_base_url,
      model: settings.model,
      default_prompt: settings.default_prompt,
      temperature: settings.temperature,
      max_tokens: settings.max_tokens
    })
    
    if (response.data) {
      ElMessage.success('设置保存成功')
    }
  } catch (error: any) {
    ElMessage.error('保存设置失败：' + (error.message || '未知错误'))
  } finally {
    saving.value = false
  }
}

onMounted(() => {
  loadSettings()
})
</script>

<style scoped src="../styles/ai-settings-view.css"></style>
