import api from './index'

// 特殊用户类型
export type SpecialUserType = 'system' | 'iot' | 'scerm' | 'sysai' | 'chatai'

// 特殊用户响应
export interface SpecialUserResponse {
  id: string
  user_type: SpecialUserType
  identifier: string
  linked_person_id?: string
  linked_person_name?: string
  description?: string
  is_active: boolean
  last_login_at?: string
  created_at: string
}

// 创建特殊用户请求
export interface CreateSpecialUserRequest {
  user_type: 'iot' | 'scerm'
  identifier: string
  description?: string
  api_key?: string
}

// 更新特殊用户请求
export interface UpdateSpecialUserRequest {
  description?: string
  is_active?: boolean
}

// 关联人员请求
export interface LinkPersonRequest {
  person_id: string
}

// 特殊用户登录请求
export interface SpecialUserLoginRequest {
  identifier: string
  api_key: string
}

// 特殊用户登录响应
export interface SpecialUserLoginResponse {
  token: string
  user_type: string
  identifier: string
  expires_in: number
}

// 操作日志响应
export interface OperationLogResponse {
  id: string
  operator_type: string
  operator_name: string
  action: string
  resource_type?: string
  resource_id?: string
  details?: Record<string, unknown>
  created_at: string
}

// 特殊用户API
export const specialUserApi = {
  // 获取特殊用户列表
  list: (params?: { user_type?: string; is_active?: boolean }) =>
    api.get<SpecialUserResponse[]>('/special-users', { params }),

  // 创建特殊用户
  create: (data: CreateSpecialUserRequest) =>
    api.post<SpecialUserResponse>('/special-users', data),

  // 更新特殊用户
  update: (id: string, data: UpdateSpecialUserRequest) =>
    api.put<SpecialUserResponse>(`/special-users/${id}`, data),

  // 删除特殊用户
  delete: (id: string) =>
    api.delete(`/special-users/${id}`),

  // 关联人员
  linkPerson: (id: string, data: LinkPersonRequest) =>
    api.post<SpecialUserResponse>(`/special-users/${id}/link`, data),

  // 特殊用户登录（IoT/Scerm）
  login: (data: SpecialUserLoginRequest) =>
    api.post<SpecialUserLoginResponse>('/special-users/login', data),

  // 获取操作日志
  getOperationLogs: (params?: { limit?: number; offset?: number }) =>
    api.get<OperationLogResponse[]>('/operation-logs', { params }),
}
