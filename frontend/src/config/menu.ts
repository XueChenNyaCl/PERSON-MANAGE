import type { MenuItem } from './types'

// 菜单项配置
export const menuItems: MenuItem[] = [
  // 首页/仪表盘
  {
    id: 'dashboard',
    title: '仪表盘',
    icon: 'House',
    path: '/dashboard',
    requiredPermission: 'dashboard.view',
    children: []
  },
  
  // 人员管理
  {
    id: 'person',
    title: '人员列表',
    icon: 'User',
    path: '/dashboard/person',
    requiredPermission: 'person.view',
    parentId: 'person-management'
  },
  {
    id: 'class',
    title: '班级列表',
    icon: 'Suitcase',
    path: '/dashboard/class',
    requiredPermission: 'class.view',
    parentId: 'person-management'
  },
  {
    id: 'class-manage',
    title: '班级管理',
    icon: 'Operation',
    path: '/dashboard/class/manage',
    requiredPermission: 'class.manage',
    parentId: 'person-management'
  },
  {
    id: 'department',
    title: '部门列表',
    icon: 'OfficeBuilding',
    path: '/dashboard/department',
    requiredPermission: 'department.view',
    parentId: 'person-management'
  },
  
  // 考勤管理
  {
    id: 'attendance',
    title: '考勤记录',
    icon: 'Timer',
    path: '/dashboard/attendance',
    requiredPermission: 'attendance.view',
    parentId: 'attendance-management'
  },
  
  // 评分管理
  {
    id: 'score',
    title: '评分记录',
    icon: 'GoodsFilled',
    path: '/dashboard/score',
    requiredPermission: 'score.view',
    parentId: 'score-management'
  },
  
  // 通知公告
  {
    id: 'notice',
    title: '通知列表',
    icon: 'Message',
    path: '/dashboard/notice',
    requiredPermission: 'notice.view',
    parentId: 'notice-management'
  },
  
  // 系统设置（仅管理员）
  {
    id: 'system-permission',
    title: '用户权限',
    icon: 'Lock',
    path: '/dashboard/system/permission',
    requiredPermission: 'system.settings',
    parentId: 'system-settings'
  },
  {
    id: 'system-plugin',
    title: '插件管理',
    icon: 'Grid',
    path: '/dashboard/system/plugin',
    requiredPermission: 'system.settings',
    parentId: 'system-settings'
  }
]

// 菜单分组
export const menuGroups = [
  {
    id: 'person-management',
    title: '人员管理'
  },
  {
    id: 'attendance-management',
    title: '考勤管理'
  },
  {
    id: 'score-management',
    title: '评分管理'
  },
  {
    id: 'notice-management',
    title: '通知公告'
  },
  {
    id: 'system-settings',
    title: '系统设置'
  }
]