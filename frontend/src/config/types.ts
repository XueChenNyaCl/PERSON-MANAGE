export interface MenuItem {
  id: string
  title: string
  icon: string
  path: string
  requiredPermission?: string
  parentId?: string
  children?: MenuItem[]
}