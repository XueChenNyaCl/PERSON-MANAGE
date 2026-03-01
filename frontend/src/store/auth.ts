import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { UserInfo } from '../api/auth'

export const useAuthStore = defineStore('auth', () => {
  const token = ref(localStorage.getItem('token') || '')
  const user = ref<UserInfo | null>(null)
  const permissions = ref<string[]>([])
  const classPermissions = ref<Record<string, string[]>>({})
  
  // 初始化时从localStorage加载用户信息
  const storedUser = localStorage.getItem('user')
  if (storedUser) {
    try {
      user.value = JSON.parse(storedUser)
    } catch (e) {
      console.error('Failed to parse stored user:', e)
    }
  }
  
  // 初始化权限
  const storedPermissions = localStorage.getItem('permissions')
  if (storedPermissions) {
    try {
      permissions.value = JSON.parse(storedPermissions)
    } catch (e) {
      console.error('Failed to parse stored permissions:', e)
    }
  }
  
  // 初始化班级特定权限
  const storedClassPermissions = localStorage.getItem('classPermissions')
  if (storedClassPermissions) {
    try {
      classPermissions.value = JSON.parse(storedClassPermissions)
    } catch (e) {
      console.error('Failed to parse stored class permissions:', e)
    }
  }
  
  const isAuthenticated = computed(() => !!token.value)
  const userRole = computed(() => user.value?.role || '')
  const userName = computed(() => user.value?.name || '')
  
  function setAuth(newToken: string, newUser: UserInfo, newPermissions: string[], newClassPermissions?: Record<string, string[]>) {
    token.value = newToken
    user.value = newUser
    permissions.value = newPermissions
    if (newClassPermissions) {
      classPermissions.value = newClassPermissions
    }
    
    localStorage.setItem('token', newToken)
    localStorage.setItem('user', JSON.stringify(newUser))
    localStorage.setItem('permissions', JSON.stringify(newPermissions))
    localStorage.setItem('classPermissions', JSON.stringify(classPermissions.value))
  }
  
  function clearAuth() {
    token.value = ''
    user.value = null
    permissions.value = []
    classPermissions.value = {}
    
    localStorage.removeItem('token')
    localStorage.removeItem('user')
    localStorage.removeItem('permissions')
    localStorage.removeItem('classPermissions')
  }
  
  function hasPermission(permission: string): boolean {
    // 检查完全匹配
    if (permissions.value.includes(permission)) {
      return true
    }
    
    // 检查通配符权限
    const firstPart = permission.split('.')[0]
    if (firstPart && permissions.value.includes(`${firstPart}.*`)) {
      return true
    }
    
    // 检查全局通配符
    if (permissions.value.includes('*')) {
      return true
    }
    
    return false
  }
  
  function hasAnyPermission(permissionList: string[]): boolean {
    return permissionList.some(permission => hasPermission(permission))
  }
  
  function hasAllPermissions(permissionList: string[]): boolean {
    return permissionList.every(permission => hasPermission(permission))
  }
  
  // 获取班级ID的后6位
  function getClassSuffix(classId: string): string {
    return classId.replace(/-/g, '').slice(-6)
  }
  
  // 检查是否拥有特定班级的权限
  function hasClassPermission(permission: string, classId: string): boolean {
    // 1. 检查通用权限（支持通配符）
    if (hasPermission(permission)) {
      return true
    }
    
    // 2. 检查班级特定权限
    const classSuffix = getClassSuffix(classId)
    const classPerms = classPermissions.value[permission]
    if (classPerms && classPerms.includes(classSuffix)) {
      return true
    }
    
    return false
  }
  
  return {
    token,
    user,
    permissions,
    classPermissions,
    isAuthenticated,
    userRole,
    userName,
    setAuth,
    clearAuth,
    hasPermission,
    hasAnyPermission,
    hasAllPermissions,
    hasClassPermission
  }
})
