import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { UserInfo } from '../api/auth'

export const useAuthStore = defineStore('auth', () => {
  const token = ref(localStorage.getItem('token') || '')
  const user = ref<UserInfo | null>(null)
  const permissions = ref<string[]>([])
  
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
  
  const isAuthenticated = computed(() => !!token.value)
  const userRole = computed(() => user.value?.role || '')
  const userName = computed(() => user.value?.name || '')
  
  function setAuth(newToken: string, newUser: UserInfo, newPermissions: string[]) {
    token.value = newToken
    user.value = newUser
    permissions.value = newPermissions
    
    localStorage.setItem('token', newToken)
    localStorage.setItem('user', JSON.stringify(newUser))
    localStorage.setItem('permissions', JSON.stringify(newPermissions))
  }
  
  function clearAuth() {
    token.value = ''
    user.value = null
    permissions.value = []
    
    localStorage.removeItem('token')
    localStorage.removeItem('user')
    localStorage.removeItem('permissions')
  }
  
  function hasPermission(permission: string): boolean {
    return permissions.value.includes(permission)
  }
  
  function hasAnyPermission(permissionList: string[]): boolean {
    return permissionList.some(permission => hasPermission(permission))
  }
  
  function hasAllPermissions(permissionList: string[]): boolean {
    return permissionList.every(permission => hasPermission(permission))
  }
  
  return {
    token,
    user,
    permissions,
    isAuthenticated,
    userRole,
    userName,
    setAuth,
    clearAuth,
    hasPermission,
    hasAnyPermission,
    hasAllPermissions
  }
})