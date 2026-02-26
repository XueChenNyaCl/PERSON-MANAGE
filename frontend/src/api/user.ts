import api from './index'

// 用户类型定义
export interface User {
  id: string
  username: string
  email?: string
  role: string
  created_at?: string
  updated_at?: string
  is_active?: boolean
}

export interface UserQuery {
  page: number
  limit: number
  search?: string
  role?: string
  is_active?: boolean
}

export interface ListResponse<T> {
  items: T[]
  total: number
  page: number
  limit: number
}

// 用户管理API
export const userApi = {
  // 获取用户列表
  list: (params: UserQuery) => {
    return api.get<ListResponse<User>>('/users', { params })
  },
  
  // 获取单个用户
  get: (id: string) => {
    return api.get<User>(`/users/${id}`)
  },
  
  // 创建用户
  create: (data: Partial<User>) => {
    return api.post<User>('/users', data)
  },
  
  // 更新用户
  update: (id: string, data: Partial<User>) => {
    return api.put<User>(`/users/${id}`, data)
  },
  
  // 删除用户
  delete: (id: string) => {
    return api.delete(`/users/${id}`)
  },
  
  // 获取用户角色
  getUserRole: (userId: string) => {
    return api.get<{ role: string }>(`/users/${userId}/role`)
  },
  
  // 更新用户角色
  updateUserRole: (userId: string, role: string) => {
    return api.put(`/users/${userId}/role`, { role })
  },
  
  // 搜索用户
  search: (query: string) => {
    return api.get<User[]>(`/users/search?q=${encodeURIComponent(query)}`)
  },
  
  // 获取用户统计
  getStats: () => {
    return api.get<{ total: number; by_role: Record<string, number> }>('/users/stats')
  }
}