import api from './index'

// 评分查询参数
export interface ScoreQuery {
  page: number
  limit: number
  person_id?: string
  group_id?: string
  score_type?: string
}

// 评分创建参数
export interface ScoreCreate {
  person_id: string
  group_id?: string
  score_type: string
  value: number
  reason: string
}

// 评分更新参数
export interface ScoreUpdate {
  value?: number
  reason?: string
}

// 评分响应数据
export interface ScoreResponse {
  id: string
  person_id: string
  person_name: string
  group_id?: string
  group_name?: string
  score_type: string
  value: number
  reason: string
  created_at: string
}

// 列表响应
export interface ListResponse<T> {
  items: T[]
  total: number
}

// 评分API
export const scoreApi = {
  list: (params: ScoreQuery) => {
    return api.get<ListResponse<ScoreResponse>>('/scores', { params })
  },
  create: (data: ScoreCreate) => {
    return api.post<ScoreResponse>('/scores', data)
  },
  get: (id: string) => {
    return api.get<ScoreResponse>(`/scores/${id}`)
  },
  update: (id: string, data: ScoreUpdate) => {
    return api.put<ScoreResponse>(`/scores/${id}`, data)
  },
  delete: (id: string) => {
    return api.delete(`/scores/${id}`)
  }
}
