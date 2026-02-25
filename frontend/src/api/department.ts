import api from './index'

// 部门类型定义
export interface DepartmentCreate {
  name: string
  parent_id?: string
}

export interface DepartmentUpdate {
  name?: string
  parent_id?: string
}

export interface DepartmentResponse {
  id: string
  name: string
  parent_id?: string
  parent_name?: string
  created_at: string
}

export interface DepartmentQuery {
  page: number
  limit: number
  search?: string
}

export interface ListResponse<T> {
  items: T[]
  total: number
  page: number
  limit: number
}

// 部门管理API
export const departmentApi = {
  // 获取部门列表
  list: (params: DepartmentQuery) => {
    return api.get<ListResponse<DepartmentResponse>>('/departments', { params })
  },
  
  // 创建部门
  create: (data: DepartmentCreate) => {
    return api.post<DepartmentResponse>('/departments', data)
  },
  
  // 获取单个部门
  get: (id: string) => {
    return api.get<DepartmentResponse>(`/departments/${id}`)
  },
  
  // 更新部门
  update: (id: string, data: DepartmentUpdate) => {
    return api.put<DepartmentResponse>(`/departments/${id}`, data)
  },
  
  // 删除部门
  delete: (id: string) => {
    return api.delete(`/departments/${id}`)
  }
}
