<template>
  <div v-if="visible" class="ai-assistant">
    <div class="ai-assistant-icon">💡</div>
    <div class="ai-assistant-content">{{ suggestion }}</div>
    <div class="ai-assistant-close" @click="close">×</div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useRoute } from 'vue-router'
import { aiApi } from '../api/ai'
import { usePageContext } from '../composables/usePageContext'
import { useAIStore } from '../store/ai'

const route = useRoute()
const aiStore = useAIStore()
const { getContext } = usePageContext()

const loading = ref(false)
const suggestion = ref('')

const visible = computed(() => {
  return !!suggestion.value && !aiStore.isDismissed(route.path)
})

const fetchSuggestion = async () => {
  if (loading.value) return

  loading.value = true
  try {
    aiStore.clearDismissed(route.path)

    const context = await getContext()
    if (!context) {
      suggestion.value = ''
      return
    }

    const response = await aiApi.getAssistantSuggestion({
      page_context: context,
      path: route.path,
      name: String(route.name || '')
    })

    suggestion.value = response.data?.suggestion || ''
  } catch (error) {
    console.error('获取AI助手建议失败:', error)
    suggestion.value = ''
  } finally {
    loading.value = false
  }
}

const close = () => {
  aiStore.dismiss(route.path)
}

watch(() => route.fullPath, () => {
  fetchSuggestion()
}, { immediate: true })
</script>

<style scoped>
@import '../styles/ai-assistant.css';
</style>
