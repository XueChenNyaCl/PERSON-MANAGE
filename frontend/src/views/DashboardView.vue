<template>
  <div class="dashboard-container">
    <!-- 侧边栏 -->
    <div class="sidebar">
      <!-- 1. 首页 / 仪表盘 -->
      <div class="sidebar-section">
        <div class="sidebar-section-title">首页 / 仪表盘</div>
        <div class="sidebar-item" :class="{ active: isActive('dashboard') }" @click="navigateTo('dashboard')">
          <div class="sidebar-icon">
            <el-icon><House /></el-icon>
          </div>
          <span>仪表盘</span>
        </div>
      </div>
      
      <!-- 2. 人员管理 -->
      <div class="sidebar-section">
        <div class="sidebar-section-title">人员管理</div>
        <div class="sidebar-item" :class="{ active: isActive('person') }" @click="navigateTo('person')">
          <div class="sidebar-icon">
            <el-icon><User /></el-icon>
          </div>
          <span>人员列表</span>
        </div>
        <div class="sidebar-item" :class="{ active: isActive('class') }" @click="navigateTo('class')">
          <div class="sidebar-icon">
            <el-icon><Suitcase /></el-icon>
          </div>
          <span>班级列表</span>
        </div>
        <div class="sidebar-item" :class="{ active: isActive('class/manage') }" @click="navigateTo('class/manage')">
          <div class="sidebar-icon">
            <el-icon><Operation /></el-icon>
          </div>
          <span>班级管理</span>
        </div>
        <div class="sidebar-item" :class="{ active: isActive('department') }" @click="navigateTo('department')">
          <div class="sidebar-icon">
            <el-icon><OfficeBuilding /></el-icon>
          </div>
          <span>部门列表</span>
        </div>
      </div>
      
      <!-- 5. 考勤管理 -->
      <div class="sidebar-section">
        <div class="sidebar-section-title">考勤管理</div>
        <div class="sidebar-item" :class="{ active: isActive('attendance/record') }" @click="navigateTo('attendance/record')">
          <div class="sidebar-icon">
            <el-icon><Timer /></el-icon>
          </div>
          <span>考勤记录</span>
        </div>
        <div class="sidebar-item" :class="{ active: isActive('attendance/stat') }" @click="navigateTo('attendance/stat')">
          <div class="sidebar-icon">
            <el-icon><DataAnalysis /></el-icon>
          </div>
          <span>考勤统计</span>
        </div>
        <div class="sidebar-item" :class="{ active: isActive('attendance/rule') }" @click="navigateTo('attendance/rule')">
          <div class="sidebar-icon">
            <el-icon><Operation /></el-icon>
          </div>
          <span>考勤规则</span>
        </div>
      </div>
      
      <!-- 4. 消费管理 -->
      <div class="sidebar-section">
        <div class="sidebar-section-title">消费管理</div>
        <div class="sidebar-item" :class="{ active: isActive('consumption/record') }" @click="navigateTo('consumption/record')">
          <div class="sidebar-icon">
            <el-icon><Money /></el-icon>
          </div>
          <span>消费记录</span>
        </div>
        <div class="sidebar-item" :class="{ active: isActive('consumption/stat') }" @click="navigateTo('consumption/stat')">
          <div class="sidebar-icon">
            <el-icon><TrendCharts /></el-icon>
          </div>
          <span>消费统计</span>
        </div>
        <div class="sidebar-item" :class="{ active: isActive('consumption/setting') }" @click="navigateTo('consumption/setting')">
          <div class="sidebar-icon">
            <el-icon><Setting /></el-icon>
          </div>
          <span>消费设置</span>
        </div>
      </div>
      
      <!-- 5. 分组管理 -->
      <div class="sidebar-section">
        <div class="sidebar-section-title">分组管理</div>
        <div class="sidebar-item" :class="{ active: isActive('group/class') }" @click="navigateTo('group/class')">
          <div class="sidebar-icon">
            <el-icon><Suitcase /></el-icon>
          </div>
          <span>班级列表</span>
        </div>
        <div class="sidebar-item" :class="{ active: isActive('group/study') }" @click="navigateTo('group/study')">
          <div class="sidebar-icon">
            <el-icon><Collection /></el-icon>
          </div>
          <span>学习小组</span>
        </div>
        <div class="sidebar-item" :class="{ active: isActive('group/dorm') }" @click="navigateTo('group/dorm')">
          <div class="sidebar-icon">
            <el-icon><House /></el-icon>
          </div>
          <span>宿舍管理</span>
        </div>
        <div class="sidebar-item" :class="{ active: isActive('group/ranking') }" @click="navigateTo('group/ranking')">
          <div class="sidebar-icon">
            <el-icon><Top /></el-icon>
          </div>
          <span>小组评分榜</span>
        </div>
      </div>
      
      <!-- 6. 评分管理 -->
      <div class="sidebar-section">
        <div class="sidebar-section-title">评分管理</div>
        <div class="sidebar-item" :class="{ active: isActive('score/record') }" @click="navigateTo('score/record')">
          <div class="sidebar-icon">
            <el-icon><GoodsFilled /></el-icon>
          </div>
          <span>评分记录</span>
        </div>
        <div class="sidebar-item" :class="{ active: isActive('score/rule') }" @click="navigateTo('score/rule')">
          <div class="sidebar-icon">
            <el-icon><Operation /></el-icon>
          </div>
          <span>评分规则</span>
        </div>
        <div class="sidebar-item" :class="{ active: isActive('score/appeal') }" @click="navigateTo('score/appeal')">
          <div class="sidebar-icon">
            <el-icon><Reading /></el-icon>
          </div>
          <span>申诉审核</span>
        </div>
      </div>
      
      <!-- 7. 评选管理 -->
      <div class="sidebar-section">
        <div class="sidebar-section-title">评选管理</div>
        <div class="sidebar-item" :class="{ active: isActive('election/initiate') }" @click="navigateTo('election/initiate')">
          <div class="sidebar-icon">
            <el-icon><Star /></el-icon>
          </div>
          <span>发起评选</span>
        </div>
        <div class="sidebar-item" :class="{ active: isActive('election/vote') }" @click="navigateTo('election/vote')">
          <div class="sidebar-icon">
            <el-icon><Check /></el-icon>
          </div>
          <span>投票管理</span>
        </div>
        <div class="sidebar-item" :class="{ active: isActive('election/result') }" @click="navigateTo('election/result')">
          <div class="sidebar-icon">
            <el-icon><Flag /></el-icon>
          </div>
          <span>评选结果</span>
        </div>
      </div>
      
      <!-- 8. 通知公告 -->
      <div class="sidebar-section">
        <div class="sidebar-section-title">通知公告</div>
        <div class="sidebar-item" :class="{ active: isActive('notice/publish') }" @click="navigateTo('notice/publish')">
          <div class="sidebar-icon">
            <el-icon><Edit /></el-icon>
          </div>
          <span>发布通知</span>
        </div>
        <div class="sidebar-item" :class="{ active: isActive('notice/list') }" @click="navigateTo('notice/list')">
          <div class="sidebar-icon">
            <el-icon><Message /></el-icon>
          </div>
          <span>通知列表</span>
        </div>
      </div>
      
      <!-- 9. 校历管理 -->
      <div class="sidebar-section">
        <div class="sidebar-section-title">校历管理</div>
        <div class="sidebar-item" :class="{ active: isActive('calendar/semester') }" @click="navigateTo('calendar/semester')">
          <div class="sidebar-icon">
            <el-icon><Calendar /></el-icon>
          </div>
          <span>学期设置</span>
        </div>
        <div class="sidebar-item" :class="{ active: isActive('calendar/holiday') }" @click="navigateTo('calendar/holiday')">
          <div class="sidebar-icon">
            <el-icon><Calendar /></el-icon>
          </div>
          <span>节假日管理</span>
        </div>
        <div class="sidebar-item" :class="{ active: isActive('calendar/view') }" @click="navigateTo('calendar/view')">
          <div class="sidebar-icon">
            <el-icon><Calendar /></el-icon>
          </div>
          <span>校历视图</span>
        </div>
      </div>
      
      <!-- 10. 活动管理 -->
      <div class="sidebar-section">
        <div class="sidebar-section-title">活动管理</div>
        <div class="sidebar-item" :class="{ active: isActive('activity/record') }" @click="navigateTo('activity/record')">
          <div class="sidebar-icon">
            <el-icon><Tickets /></el-icon>
          </div>
          <span>活动记录</span>
        </div>
        <div class="sidebar-item" :class="{ active: isActive('activity/score') }" @click="navigateTo('activity/score')">
          <div class="sidebar-icon">
            <el-icon><Trophy /></el-icon>
          </div>
          <span>活动加分</span>
        </div>
      </div>
      
      <!-- 11. 系统设置 -->
      <div class="sidebar-section">
        <div class="sidebar-section-title">系统设置</div>
        <div class="sidebar-item" :class="{ active: isActive('system/permission') }" @click="navigateTo('system/permission')">
          <div class="sidebar-icon">
            <el-icon><Lock /></el-icon>
          </div>
          <span>用户权限</span>
        </div>
        <div class="sidebar-item" :class="{ active: isActive('system/plugin') }" @click="navigateTo('system/plugin')">
          <div class="sidebar-icon">
            <el-icon><Grid /></el-icon>
          </div>
          <span>插件管理</span>
        </div>
        <div class="sidebar-item" :class="{ active: isActive('system/log') }" @click="navigateTo('system/log')">
          <div class="sidebar-icon">
            <el-icon><Document /></el-icon>
          </div>
          <span>日志查看</span>
        </div>
        <div class="sidebar-item" :class="{ active: isActive('system/config') }" @click="navigateTo('system/config')">
          <div class="sidebar-icon">
            <el-icon><Setting /></el-icon>
          </div>
          <span>系统配置</span>
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
import { hasPermission } from '../utils/permission'

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
const isAdmin = computed(() => userRole.value === 'admin')

// 过滤后的菜单项（根据权限）
const filteredMenuGroups = computed(() => {
  return menuGroups.filter(group => {
    // 获取该分组下的所有菜单项
    const groupItems = menuItems.filter(item => item.parentId === group.id)
    // 如果分组下有至少一个菜单项用户有权限访问，则显示该分组
    return groupItems.some(item => authStore.hasPermission(item.requiredPermission))
  })
})

const filteredMenuItems = computed(() => {
  return menuItems.filter(item => authStore.hasPermission(item.requiredPermission))
})

// 根据图标名称获取组件
const getIconComponent = (iconName: string) => {
  return iconComponents[iconName as keyof typeof iconComponents] || House
}

const navigateTo = (path: string) => {
  router.push(`/dashboard/${path}`)
}

const isActive = (path: string) => {
  return route.path === `/dashboard/${path}` || route.path === `/dashboard/${path}/` || (path === 'dashboard' && route.path === '/dashboard')
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
