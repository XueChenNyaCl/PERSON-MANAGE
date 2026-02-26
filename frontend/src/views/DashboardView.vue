<template>
  <div class="dashboard-container">
    <!-- 侧边栏 -->
    <!-- 侧边栏 -->
    <div class="sidebar">
      <!-- 仪表盘（独立菜单项，没有分组） -->
      <div class="sidebar-section" v-if="dashboardMenuItem">
        <div class="sidebar-section-title">首页 / 仪表盘</div>
        <div 
          class="sidebar-item" 
          :class="{ active: isDashboardActive }" 
          @click="navigateToDashboard"
        >
          <div class="sidebar-icon">
            <el-icon><component :is="getIconComponent(dashboardMenuItem.icon)" /></el-icon>
          </div>
          <span>{{ dashboardMenuItem.title }}</span>
        </div>
      </div>
      
      <!-- 动态菜单分组 -->
      <div v-for="group in filteredMenuGroups" :key="group.id" class="sidebar-section">
        <div class="sidebar-section-title">{{ group.title }}</div>
        <div
          v-for="item in getMenuItemsByGroup(group.id)"
          :key="item.id"
          class="sidebar-item"
          :class="{ active: isMenuItemActive(item) }"
          @click="navigateToMenuItem(item)"
        >
          <div class="sidebar-icon">
            <el-icon><component :is="getIconComponent(item.icon)" /></el-icon>
          </div>
          <span>{{ item.title }}</span>
        </div>
      </div>
    </div>
    
    <!-- 主内容区 -->
    <div class="main-content">
      <!-- 顶部导航栏 -->
      <div class="header">
        <div class="header-left">
          <div class="logo">SCHOOL MANAGE</div>
        </div>
        <div class="header-right">
          <div class="header-icon">
            <el-icon><Bell /></el-icon>
          </div>
          <div class="header-icon">
            <el-icon><Setting /></el-icon>
          </div>
          <div class="user-avatar" @click="toggleUserMenu">
            <el-avatar size="small">{{ username.charAt(0) }}</el-avatar>
          </div>
          
          <!-- 用户下拉菜单 -->
          <div v-if="userMenuVisible" class="user-menu">
            <div class="user-menu-item" @click="handleLogout">退出登录</div>
          </div>
        </div>
      </div>
      
      <!-- 内容区域 -->
      <div class="content">
        <!-- 检查是否是仪表盘根路径 -->
        <div v-if="route.path === '/dashboard'" class="dashboard-cards">
          <h2 class="dashboard-title">仪表盘</h2>
          
          <!-- 统计卡片 -->
          <div class="stats-cards">
            <div class="stat-card">
              <div class="stat-icon">
                <el-icon><User /></el-icon>
              </div>
              <div class="stat-content">
                <div class="stat-number">0</div>
                <div class="stat-label">总人数</div>
              </div>
            </div>
            <div class="stat-card">
              <div class="stat-icon">
                <el-icon><Timer /></el-icon>
              </div>
              <div class="stat-content">
                <div class="stat-number">0</div>
                <div class="stat-label">今日考勤</div>
              </div>
            </div>
            <div class="stat-card">
              <div class="stat-icon">
                <el-icon><Ticket /></el-icon>
              </div>
              <div class="stat-content">
                <div class="stat-number">0</div>
                <div class="stat-label">待办事项</div>
              </div>
            </div>
            <div class="stat-card">
              <div class="stat-icon">
                <el-icon><Top /></el-icon>
              </div>
              <div class="stat-content">
                <div class="stat-number">0</div>
                <div class="stat-label">分数排行</div>
              </div>
            </div>
          </div>
          
          <!-- 图表卡片 -->
          <div class="chart-cards">
            <div class="chart-card">
              <div class="card-header">
                <h3>考勤统计</h3>
              </div>
              <div class="card-body">
                <!-- 图表占位 -->
                <div class="chart-placeholder"></div>
              </div>
            </div>
            <div class="chart-card">
              <div class="card-header">
                <h3>评分趋势</h3>
              </div>
              <div class="card-body">
                <!-- 图表占位 -->
                <div class="chart-placeholder"></div>
              </div>
            </div>
          </div>
          
          <!-- 最近动态 -->
          <div class="activity-card">
            <div class="card-header">
              <h3>最近动态</h3>
            </div>
            <div class="card-body">
              <div class="activity-list">
                <!-- 动态项占位 -->
                <div class="activity-item">
                  <div class="activity-icon"></div>
                  <div class="activity-content">
                    <div class="activity-title">动态标题</div>
                    <div class="activity-time">时间</div>
                  </div>
                </div>
                <div class="activity-item">
                  <div class="activity-icon"></div>
                  <div class="activity-content">
                    <div class="activity-title">动态标题</div>
                    <div class="activity-time">时间</div>
                  </div>
                </div>
                <div class="activity-item">
                  <div class="activity-icon"></div>
                  <div class="activity-content">
                    <div class="activity-title">动态标题</div>
                    <div class="activity-time">时间</div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
        
        <!-- 其他路由内容 -->
        <router-view v-else />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { 
  House, User, UserFilled, Timer, GoodsFilled, Message, Bell, Setting, 
  Suitcase, OfficeBuilding, DataAnalysis, Operation, Money, TrendCharts, 
  Collection, Top, Reading, Star, Check, Flag, Edit, Calendar, 
  Tickets, Trophy, Lock, Grid, Document, Ticket 
} from '@element-plus/icons-vue'
import { useAuthStore } from '../store/auth'
import { menuItems, menuGroups } from '../config/menu'
import type { MenuItem } from '../config/types'

// 图标名称到组件的映射
const iconComponents = {
  House,
  User,
  UserFilled,
  Timer,
  GoodsFilled,
  Message,
  Bell,
  Setting,
  Suitcase,
  OfficeBuilding,
  DataAnalysis,
  Operation,
  Money,
  TrendCharts,
  Collection,
  Top,
  Reading,
  Star,
  Check,
  Flag,
  Edit,
  Calendar,
  Tickets,
  Trophy,
  Lock,
  Grid,
  Document,
  Ticket
}

const router = useRouter()
const route = useRoute()
const authStore = useAuthStore()
const userMenuVisible = ref(false)

const username = computed(() => authStore.userName || '管理员')
const userRole = computed(() => authStore.userRole || '')

// 过滤后的菜单项（根据权限）
const filteredMenuGroups = computed(() => {
  return menuGroups.filter(group => {
    // 获取该分组下的所有菜单项
    const groupItems = menuItems.filter(item => item.parentId === group.id)
    // 如果分组下有至少一个菜单项用户有权限访问，则显示该分组
    return groupItems.some(item => {
      if (!item.requiredPermission) return true // 不需要权限
      return authStore.hasPermission(item.requiredPermission)
    })
  })
})

const filteredMenuItems = computed(() => {
  return menuItems.filter(item => {
    if (!item.requiredPermission) return true // 不需要权限
    return authStore.hasPermission(item.requiredPermission)
  })
})

// 仪表盘菜单项（没有分组的独立菜单项）
const dashboardMenuItem = computed(() => {
  return menuItems.find(item => {
    if (!item.parentId) {
      if (!item.requiredPermission) return true // 不需要权限
      return authStore.hasPermission(item.requiredPermission)
    }
    return false
  })
})

// 仪表盘是否激活
const isDashboardActive = computed(() => {
  return route.path === '/dashboard'
})

// 根据分组ID获取菜单项
const getMenuItemsByGroup = (groupId: string) => {
  return filteredMenuItems.value.filter(item => item.parentId === groupId)
}

// 检查菜单项是否激活
const isMenuItemActive = (item: MenuItem) => {
  if (!item.path) return false
  return route.path === item.path || route.path === `${item.path}/`
}

// 导航到仪表盘
const navigateToDashboard = () => {
  router.push('/dashboard')
}

// 导航到菜单项
const navigateToMenuItem = (item: MenuItem) => {
  if (item.path) {
    router.push(item.path)
  }
}

// 根据图标名称获取组件
const getIconComponent = (iconName: string) => {
  return iconComponents[iconName as keyof typeof iconComponents] || House
}



const toggleUserMenu = () => {
  userMenuVisible.value = !userMenuVisible.value
}

const handleLogout = () => {
  authStore.clearAuth()
  router.push('/login')
}

onMounted(() => {
  // 检查是否有token
  if (!authStore.isAuthenticated) {
    router.push('/login')
  }
  
  // 点击外部关闭用户菜单
  document.addEventListener('click', (e) => {
    const target = e.target as HTMLElement
    if (!target.closest('.user-avatar') && !target.closest('.user-menu')) {
      userMenuVisible.value = false
    }
  })
})
</script>

<style scoped>
.dashboard-container {
  display: flex;
  width: 100%;
  height: 100vh;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
}

/* 侧边栏样式 */
.sidebar {
  width: 200px;
  background-color: #fff;
  box-shadow: 0 0 10px rgba(0, 0, 0, 0.05);
  padding-top: 60px;
  height: 100vh;
  position: fixed;
  left: 0;
  top: 0;
  overflow-y: auto;
}

.sidebar-section {
  margin-bottom: 20px;
}

.sidebar-section-title {
  font-size: 12px;
  font-weight: 500;
  color: #999;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  margin: 16px 16px 8px;
  padding: 0 8px;
}

.sidebar-item {
  display: flex;
  align-items: center;
  height: 40px;
  padding: 0 16px;
  cursor: pointer;
  transition: all 0.3s ease;
  color: #4e5969;
  border-radius: 2px;
  margin: 0 4px;
}

.sidebar-item:hover {
  background-color: #f2f3f5;
  color: #165dff;
}

.sidebar-item.active {
  background-color: #f2f3f5;
  color: #165dff;
  font-weight: 500;
}

.sidebar-icon {
  margin-right: 16px;
  font-size: 18px;
}

/* 主内容区样式 */
.main-content {
  flex: 1;
  margin-left: 200px;
  height: 100vh;
  display: flex;
  flex-direction: column;
}

/* 顶部导航栏样式 */
.header {
  height: 60px;
  background-color: #fff;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.06);
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 20px;
  position: fixed;
  top: 0;
  right: 0;
  left: 200px;
  z-index: 100;
}

.header-left {
  display: flex;
  align-items: center;
}

.logo {
  font-size: 20px;
  font-weight: 500;
  color: #165dff;
  letter-spacing: 1px;
}

.header-right {
  display: flex;
  align-items: center;
  gap: 20px;
  position: relative;
}

.header-icon {
  font-size: 20px;
  color: #4e5969;
  cursor: pointer;
  transition: color 0.3s ease;
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
}

.header-icon:hover {
  color: #165dff;
  background-color: #f2f3f5;
}

.user-avatar {
  cursor: pointer;
  position: relative;
}

/* 用户下拉菜单 */
.user-menu {
  position: absolute;
  top: 40px;
  right: 0;
  background-color: #fff;
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.1);
  border-radius: 4px;
  min-width: 120px;
  z-index: 101;
}

.user-menu-item {
  padding: 8px 16px;
  cursor: pointer;
  transition: background-color 0.3s ease;
}

.user-menu-item:hover {
  background-color: #f2f3f5;
}

/* 内容区域 */
.content {
  flex: 1;
  padding: 20px;
  margin-top: 60px;
  background-color: #f5f5f5;
  overflow-y: auto;
}

/* 仪表盘样式 */
.dashboard-title {
  font-size: 24px;
  font-weight: 500;
  color: #165dff;
  margin-bottom: 20px;
  letter-spacing: 1px;
}

/* 统计卡片 */
.stats-cards {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 20px;
  margin-bottom: 30px;
}

.stat-card {
  background-color: #fff;
  border-radius: 8px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.06);
  padding: 20px;
  display: flex;
  align-items: center;
  transition: transform 0.3s ease, box-shadow 0.3s ease;
}

.stat-card:hover {
  transform: translateY(-5px);
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.1);
}

.stat-icon {
  width: 50px;
  height: 50px;
  border-radius: 50%;
  background-color: #f2f3f5;
  display: flex;
  align-items: center;
  justify-content: center;
  margin-right: 16px;
  font-size: 24px;
  color: #165dff;
}

.stat-content {
  flex: 1;
}

.stat-number {
  font-size: 28px;
  font-weight: 500;
  color: #161616;
  margin-bottom: 4px;
}

.stat-label {
  font-size: 14px;
  color: #787878;
}

/* 图表卡片 */
.chart-cards {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(400px, 1fr));
  gap: 20px;
  margin-bottom: 30px;
}

.chart-card {
  background-color: #fff;
  border-radius: 8px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.06);
  overflow: hidden;
}

.card-header {
  padding: 16px 20px;
  border-bottom: 1px solid #f0f0f0;
}

.card-header h3 {
  font-size: 16px;
  font-weight: 500;
  color: #161616;
  margin: 0;
}

.card-body {
  padding: 20px;
}

.chart-placeholder {
  width: 100%;
  height: 300px;
  background-color: #f9f9f9;
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #999;
}

/* 最近动态 */
.activity-card {
  background-color: #fff;
  border-radius: 8px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.06);
  overflow: hidden;
}

.activity-list {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.activity-item {
  display: flex;
  align-items: flex-start;
  padding-bottom: 16px;
  border-bottom: 1px solid #f0f0f0;
}

.activity-item:last-child {
  border-bottom: none;
  padding-bottom: 0;
}

.activity-icon {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  background-color: #f2f3f5;
  margin-right: 12px;
  flex-shrink: 0;
}

.activity-content {
  flex: 1;
}

.activity-title {
  font-size: 14px;
  color: #161616;
  margin-bottom: 4px;
}

.activity-time {
  font-size: 12px;
  color: #999;
}

/* 响应式设计 */
@media (max-width: 768px) {
  .sidebar {
    width: 60px;
  }
  
  .sidebar-item span {
    display: none;
  }
  
  .sidebar-icon {
    margin-right: 0;
  }
  
  .main-content {
    margin-left: 60px;
  }
  
  .header {
    left: 60px;
  }
  
  .logo {
    font-size: 16px;
  }
  
  .stats-cards {
    grid-template-columns: 1fr;
  }
  
  .chart-cards {
    grid-template-columns: 1fr;
  }
  
  .chart-card {
    min-width: unset;
  }
  
  .chart-placeholder {
    height: 200px;
  }
}
</style>
