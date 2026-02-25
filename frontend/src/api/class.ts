import api from './index'

// 班级类型定义
export interface ClassCreate {
  name: string
  grade: number
  teacher_id?: string
  academic_year: string
}

export interface ClassUpdate {
  name?: string
  grade?: number
  teacher_id?: string
  academic_year?: string
}

export interface ClassResponse {
  id: string
  name: string
  grade: number
  teacher_id?: string
  teacher_name?: string
  academic_year: string
  created_at: string
}

export interface ClassQuery {
  page: number
  limit: number
  search?: string
  grade?: number
}

export interface ListResponse<T> {
  items: T[]
  total: number
  page: number
  limit: number
}

// 班级管理API
export const classApi = {
  // 获取班级列表
  list: (params: ClassQuery) => {
    return api.get<ListResponse<ClassResponse>>('/classes', { params })
  },
  
  // 创建班级
  create: (data: ClassCreate) => {
    return api.post<ClassResponse>('/classes', data)
  },
  
  // 获取单个班级
  get: (id: string) => {
    return api.get<ClassResponse>(`/classes/${id}`)
  },
  
  // 更新班级
  update: (id: string, data: ClassUpdate) => {
    return api.put<ClassResponse>(`/classes/${id}`, data)
  },
  
  // 删除班级
  delete: (id: string) => {
    return api.delete(`/classes/${id}`)
  }
}
