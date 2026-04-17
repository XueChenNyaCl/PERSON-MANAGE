import api from './index'

export interface DatabaseStatus {
  connected: boolean
  status: string
  message: string
  response_time_ms?: number
}

export interface RedisMemoryInfo {
  used_memory: number
  max_memory: number
  usage_percent: number
  used_memory_human: string
  max_memory_human: string
}

export interface RedisStatsInfo {
  connected_clients: number
  total_commands_processed: number
  keyspace_hits: number
  keyspace_misses: number
  hit_rate: number
  uptime_in_seconds: number
}

export interface RedisStatus {
  connected: boolean
  status: string
  message: string
  memory?: RedisMemoryInfo
  stats?: RedisStatsInfo
}

export interface MonitorStatusResponse {
  postgresql: DatabaseStatus
  redis: RedisStatus
  timestamp: string
}

export interface BufferStatusResponse {
  enabled: boolean
  queue_length: number
  last_flush_time?: string
  total_flushed: number
  total_failed: number
}

export interface FlushBufferResponse {
  success: boolean
  message: string
  flushed_count?: number
}

export const monitorApi = {
  getStatus() {
    return api.get<MonitorStatusResponse>('/monitor/status')
  },

  getBufferStatus() {
    return api.get<BufferStatusResponse>('/monitor/buffer')
  },

  flushBuffer() {
    return api.post<FlushBufferResponse>('/monitor/buffer/flush', { confirm: true })
  }
}
