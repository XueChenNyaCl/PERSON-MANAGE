<template>
  <div class="monitor-view">
    <div class="monitor-header">
      <h1 class="monitor-title">系统监控</h1>
      <p class="monitor-subtitle">实时监控 PostgreSQL 和 Redis 连接状态</p>
    </div>

    <div class="monitor-grid">
      <!-- PostgreSQL 状态卡片 -->
      <div class="monitor-card">
        <div class="card-header">
          <div class="card-icon postgresql">
            <el-icon><DataLine /></el-icon>
          </div>
          <div class="card-title">PostgreSQL</div>
          <div 
            class="status-badge"
            :class="{ 'status-online': status?.postgresql?.connected, 'status-offline': !status?.postgresql?.connected }"
          >
            {{ status?.postgresql?.connected ? '在线' : '离线' }}
          </div>
        </div>
        <div class="card-content">
          <div class="info-row">
            <span class="info-label">状态:</span>
            <span class="info-value" :class="{ 'text-success': status?.postgresql?.connected, 'text-error': !status?.postgresql?.connected }">
              {{ status?.postgresql?.status || 'unknown' }}
            </span>
          </div>
          <div class="info-row">
            <span class="info-label">消息:</span>
            <span class="info-value">{{ status?.postgresql?.message || 'N/A' }}</span>
          </div>
          <div class="info-row" v-if="status?.postgresql?.response_time_ms">
            <span class="info-label">响应时间:</span>
            <span class="info-value">{{ status.postgresql.response_time_ms }}ms</span>
          </div>
        </div>
      </div>

      <!-- Redis 状态卡片 -->
      <div class="monitor-card">
        <div class="card-header">
          <div class="card-icon redis">
            <el-icon><Collection /></el-icon>
          </div>
          <div class="card-title">Redis</div>
          <div 
            class="status-badge"
            :class="{ 'status-online': status?.redis?.connected, 'status-offline': !status?.redis?.connected }"
          >
            {{ status?.redis?.connected ? '在线' : '离线' }}
          </div>
        </div>
        <div class="card-content">
          <div class="info-row">
            <span class="info-label">状态:</span>
            <span class="info-value" :class="{ 'text-success': status?.redis?.connected, 'text-error': !status?.redis?.connected }">
              {{ status?.redis?.status || 'unknown' }}
            </span>
          </div>
          <div class="info-row">
            <span class="info-label">消息:</span>
            <span class="info-value">{{ status?.redis?.message || 'N/A' }}</span>
          </div>
        </div>
      </div>

      <!-- Redis 内存信息卡片 -->
      <div class="monitor-card" v-if="status?.redis?.memory">
        <div class="card-header">
          <div class="card-icon memory">
            <el-icon><Cpu /></el-icon>
          </div>
          <div class="card-title">Redis 内存使用</div>
        </div>
        <div class="card-content">
          <div class="memory-progress">
            <div class="progress-bar">
              <div 
                class="progress-fill"
                :style="{ width: status.redis.memory.usage_percent + '%' }"
                :class="{ 'progress-warning': status.redis.memory.usage_percent > 80, 'progress-danger': status.redis.memory.usage_percent > 90 }"
              ></div>
            </div>
            <div class="progress-text">{{ status.redis.memory.usage_percent.toFixed(1) }}%</div>
          </div>
          <div class="info-row">
            <span class="info-label">已使用:</span>
            <span class="info-value">{{ status.redis.memory.used_memory_human }}</span>
          </div>
          <div class="info-row">
            <span class="info-label">最大限制:</span>
            <span class="info-value">{{ status.redis.memory.max_memory_human }}</span>
          </div>
        </div>
      </div>

      <!-- Redis 统计信息卡片 -->
      <div class="monitor-card" v-if="status?.redis?.stats">
        <div class="card-header">
          <div class="card-icon stats">
            <el-icon><TrendCharts /></el-icon>
          </div>
          <div class="card-title">Redis 统计</div>
        </div>
        <div class="card-content">
          <div class="stats-grid">
            <div class="stat-item">
              <div class="stat-value">{{ status.redis.stats.connected_clients }}</div>
              <div class="stat-label">连接客户端</div>
            </div>
            <div class="stat-item">
              <div class="stat-value">{{ formatNumber(status.redis.stats.total_commands_processed) }}</div>
              <div class="stat-label">命令处理数</div>
            </div>
            <div class="stat-item">
              <div class="stat-value">{{ status.redis.stats.hit_rate.toFixed(1) }}%</div>
              <div class="stat-label">缓存命中率</div>
            </div>
            <div class="stat-item">
              <div class="stat-value">{{ formatUptime(status.redis.stats.uptime_in_seconds) }}</div>
              <div class="stat-label">运行时间</div>
            </div>
          </div>
        </div>
      </div>

      <!-- 写入缓冲状态卡片 -->
      <div class="monitor-card">
        <div class="card-header">
          <div class="card-icon buffer">
            <el-icon><Timer /></el-icon>
          </div>
          <div class="card-title">写入缓冲</div>
          <div 
            class="status-badge"
            :class="{ 'status-online': bufferStatus?.enabled, 'status-offline': !bufferStatus?.enabled }"
          >
            {{ bufferStatus?.enabled ? '启用' : '禁用' }}
          </div>
        </div>
        <div class="card-content">
          <div class="info-row">
            <span class="info-label">队列长度:</span>
            <span class="info-value">{{ bufferStatus?.queue_length || 0 }}</span>
          </div>
          <div class="info-row" v-if="bufferStatus?.last_flush_time">
            <span class="info-label">上次刷盘:</span>
            <span class="info-value">{{ formatTime(bufferStatus.last_flush_time) }}</span>
          </div>
          <div class="info-row" v-if="bufferStatus?.total_flushed">
            <span class="info-label">累计写入:</span>
            <span class="info-value">{{ bufferStatus.total_flushed }}</span>
          </div>
          <div class="card-actions">
            <el-button 
              type="primary" 
              size="small"
              :loading="flushing"
              :disabled="!bufferStatus?.enabled || bufferStatus?.queue_length === 0"
              @click="handleFlushBuffer"
            >
              立即刷盘
            </el-button>
          </div>
        </div>
      </div>
    </div>

    <div class="monitor-footer">
      <div class="last-update">
        最后更新: {{ formatTime(status?.timestamp) }}
      </div>
      <el-button 
        type="primary" 
        :loading="loading"
        @click="fetchStatus"
      >
        <el-icon><Refresh /></el-icon>
        刷新状态
      </el-button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { ElMessage } from 'element-plus'
import { DataLine, Collection, Cpu, TrendCharts, Timer, Refresh } from '@element-plus/icons-vue'
import { monitorApi } from '../api/monitor'

interface DatabaseStatus {
  connected: boolean
  status: string
  message: string
  response_time_ms?: number
}

interface RedisMemoryInfo {
  used_memory: number
  max_memory: number
  usage_percent: number
  used_memory_human: string
  max_memory_human: string
}

interface RedisStatsInfo {
  connected_clients: number
  total_commands_processed: number
  keyspace_hits: number
  keyspace_misses: number
  hit_rate: number
  uptime_in_seconds: number
}

interface RedisStatus {
  connected: boolean
  status: string
  message: string
  memory?: RedisMemoryInfo
  stats?: RedisStatsInfo
}

interface MonitorStatus {
  postgresql: DatabaseStatus
  redis: RedisStatus
  timestamp: string
}

interface BufferStatus {
  enabled: boolean
  queue_length: number
  last_flush_time?: string
  total_flushed: number
  total_failed: number
}

const status = ref<MonitorStatus | null>(null)
const bufferStatus = ref<BufferStatus | null>(null)
const loading = ref(false)
const flushing = ref(false)
let refreshTimer: number | null = null

const fetchStatus = async () => {
  loading.value = true
  try {
    const [statusRes, bufferRes] = await Promise.all([
      monitorApi.getStatus(),
      monitorApi.getBufferStatus()
    ])
    status.value = statusRes.data
    bufferStatus.value = bufferRes.data
  } catch (error) {
    ElMessage.error('获取监控状态失败')
    console.error(error)
  } finally {
    loading.value = false
  }
}

const handleFlushBuffer = async () => {
  flushing.value = true
  try {
    const res = await monitorApi.flushBuffer()
    if (res.data.success) {
      ElMessage.success('缓冲刷盘成功')
      await fetchStatus()
    } else {
      ElMessage.error(res.data.message)
    }
  } catch (error) {
    ElMessage.error('缓冲刷盘失败')
    console.error(error)
  } finally {
    flushing.value = false
  }
}

const formatNumber = (num: number): string => {
  if (num >= 1000000) {
    return (num / 1000000).toFixed(1) + 'M'
  } else if (num >= 1000) {
    return (num / 1000).toFixed(1) + 'K'
  }
  return num.toString()
}

const formatUptime = (seconds: number): string => {
  const days = Math.floor(seconds / 86400)
  const hours = Math.floor((seconds % 86400) / 3600)
  const minutes = Math.floor((seconds % 3600) / 60)
  
  if (days > 0) {
    return `${days}天${hours}小时`
  } else if (hours > 0) {
    return `${hours}小时${minutes}分钟`
  } else {
    return `${minutes}分钟`
  }
}

const formatTime = (time: string | undefined): string => {
  if (!time) return 'N/A'
  const date = new Date(time)
  return date.toLocaleString('zh-CN')
}

onMounted(() => {
  fetchStatus()
  refreshTimer = window.setInterval(fetchStatus, 30000)
})

onUnmounted(() => {
  if (refreshTimer) {
    clearInterval(refreshTimer)
  }
})
</script>

<style scoped>
@import '../styles/monitor-view.css';
</style>
