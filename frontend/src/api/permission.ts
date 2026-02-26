import api from './index'

// 权限类型定义
export interface Permission {
  permission: string
  priority: number
  created_at?: string
}

export interface UserPermissionItem {
  permission: string
  value: boolean
  priority: number
  created_at?: string
}

export interface UserPermissionListResponse {
  user_id: string
  permissions: UserPermissionItem[]
}

export interface RolePermission {
  role: string
  permission: string
  priority: number
  created_at?: string
}

export interface PermissionQuery {
  page?: number
  limit?: number
  user_id?: string
  role?: string
  search?: string
}

export interface ListResponse<T> {
  items: T[]
  total: number
  page: number
  limit: number
}

export interface YamlApplyRequest {
  yaml_content: string
  target_type: 'user' | 'role' | 'all'
  target_ids?: string[]
  role?: string
  merge_strategy: 'overwrite' | 'merge'
}

export interface PermissionTranslation {
  permission_key: string
  translation: string
}

export interface PermissionKeysResponse {
  keys: string[]
}

export interface YamlApplyResponse {
  success: boolean
  message: string
  applied_count: number
}

// 权限管理API
export const permissionApi = {
  // 获取用户权限列表
  getUserPermissions: async (userId: string): Promise<Permission[]> => {
    const response = await api.get<UserPermissionListResponse>(`/permissions/users/${userId}`)
    // 转换 UserPermissionItem 到 Permission（忽略 value 字段）
    return response.data.permissions.map(item => ({
      permission: item.permission,
      priority: item.priority,
      created_at: item.created_at
    }))
  },
  
  // 获取角色权限列表（暂时返回所有角色权限）
  getRolePermissions: (role: string) => {
    return api.get<Permission[]>(`/permissions/role/${role}`)
  },
  
  // 添加用户权限
  addUserPermission: (userId: string, permission: string, priority: number) => {
    return api.post(`/permissions/users/${userId}`, { 
      permission, 
      value: true, 
      priority 
    })
  },
  
  // 移除用户权限
  removeUserPermission: (userId: string, permission: string) => {
    return api.delete(`/permissions/users/${userId}`, {
      params: { permission }
    })
  },
  
  // 添加角色权限
  addRolePermission: (role: string, permission: string, priority: number) => {
    return api.post(`/permissions`, { 
      role, 
      permission, 
      priority 
    })
  },
  
  // 移除角色权限
  removeRolePermission: (role: string, permission: string) => {
    return api.delete(`/permissions`, {
      params: { role, permission }
    })
  },
  
  // 检查用户权限
  checkUserPermission: (userId: string, permission: string) => {
    return api.get<{ has_permission: boolean }>(`/permissions/check/user/${userId}/${encodeURIComponent(permission)}`)
  },
  
  // 检查角色权限
  checkRolePermission: (role: string, permission: string) => {
    return api.get<{ has_permission: boolean }>(`/permissions/check/role/${role}/${encodeURIComponent(permission)}`)
  },
  
  // 应用YAML模板
  applyYamlTemplate: (targetId: string, yamlContent: string, mergeStrategy: string) => {
    return api.post<YamlApplyResponse>('/permissions/apply-yaml', {
      yaml_content: yamlContent,
      target_type: 'user',
      target_ids: [targetId],
      merge_strategy: mergeStrategy
    })
  },
  
  // 批量应用YAML模板
  applyYamlTemplateBulk: (request: YamlApplyRequest) => {
    return api.post<YamlApplyResponse>('/permissions/apply-yaml', request)
  },
  
  // 获取权限翻译
  getPermissionTranslations: (permissionKeys: string[]) => {
    return api.post<PermissionTranslation[]>('/permissions/translations', { permissions: permissionKeys })
  },
  
  // 获取所有权限键
  getAllPermissionKeys: () => {
    return api.get<PermissionKeysResponse>('/permissions/keys')
  },
  
  // 获取有效权限（考虑通配符和否定权限）
  getEffectivePermissions: (userId: string) => {
    return api.get<Permission[]>(`/permissions/effective/${userId}`)
  }
}