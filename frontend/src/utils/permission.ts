import { useAuthStore } from '../store/auth'

// 权限检查函数
export function checkPermission(permission: string): boolean {
  const authStore = useAuthStore()
  return authStore.hasPermission(permission)
}

export function checkAnyPermission(permissions: string[]): boolean {
  const authStore = useAuthStore()
  return authStore.hasAnyPermission(permissions)
}

export function checkAllPermissions(permissions: string[]): boolean {
  const authStore = useAuthStore()
  return authStore.hasAllPermissions(permissions)
}

// 基于角色的权限检查
export function checkRole(role: string): boolean {
  const authStore = useAuthStore()
  return authStore.userRole === role
}

// 路由守卫权限检查
export function checkRoutePermission(requiredPermissions: string[] | null, requiredRole: string | null): boolean {
  if (!requiredPermissions && !requiredRole) {
    return true // 不需要权限
  }
  
  const authStore = useAuthStore()
  
  // 检查角色
  if (requiredRole && authStore.userRole !== requiredRole) {
    return false
  }
  
  // 检查权限
  if (requiredPermissions && !authStore.hasAnyPermission(requiredPermissions)) {
    return false
  }
  
  return true
}