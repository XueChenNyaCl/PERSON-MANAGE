import api from './index'

// 通知查询参数
export interface NoticeQuery {
  page: number
  limit: number
  target_type?: string
  target_id?: string
  search?: string
}

// 通知创建参数
export interface NoticeCreate {
  title: string
  content: string
  target_type: string
  target_id?: string
  is_important?: boolean
}

// 通知更新参数
export interface NoticeUpdate {
  title?: string
  content?: string
  is_important?: boolean
}

// 通知响应数据
export interface NoticeResponse {
  id: string
  title: string
  content: string
  author_id: string
  author_name: string
  target_type: string
  target_id?: string
  is_important: boolean
  created_at: string
}

// 列表响应
export interface ListResponse<T> {
  items: T[]
  total: number
}

// 通知API
export const noticeApi = {
  list: (params: NoticeQuery) => {
    return api.get<ListResponse<NoticeResponse>>('/notices', { params })
  },
  create: (data: NoticeCreate) => {
    return api.post<NoticeResponse>('/notices', data)
  },
  get: (id: string) => {
    return api.get<NoticeResponse>(`/notices/${id}`)
  },
  update: (id: string, data: NoticeUpdate) => {
    return api.put<NoticeResponse>(`/notices/${id}`, data)
  },
  delete: (id: string) => {
    return api.delete(`/notices/${id}`)
  }
}
