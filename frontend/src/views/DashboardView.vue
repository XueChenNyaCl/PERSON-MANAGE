<template>
  <div class="dashboard-container">
    <!-- 侧边栏 -->
    <div class="sidebar-wrapper">
      <div class="sidebar" ref="sidebarRef">
        <!-- 滑动背景 - 固定定位，不跟随滚动 -->
        <div 
          class="sidebar-active-bg" 
          :style="activeBgStyle"
          v-if="activeBgStyle"
        ></div>
        
        <!-- 仪表盘（独立菜单项，没有分组） -->
        <div class="sidebar-section" v-if="dashboardMenuItem">
          <div class="sidebar-section-title">首页 / 仪表盘</div>
          <div 
            class="sidebar-item" 
            :class="{ active: isDashboardActive }" 
            @click="navigateToDashboard"
            ref="dashboardItemRef"
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
            :ref="el => setMenuItemRef(el, item.id)"
          >
            <div class="sidebar-icon">
              <el-icon><component :is="getIconComponent(item.icon)" /></el-icon>
            </div>
            <span>{{ item.title }}</span>
          </div>
        </div>
        
      </div>
    </div>
    
    <!-- 回到顶部按钮 - 固定在页面上，位于侧边栏上方 -->
    <button 
      class="scroll-to-top-btn" 
      :class="{ visible: showScrollToTop }"
      @click="scrollToTop" 
      title="回到顶部"
    >
      <el-icon><Top /></el-icon>
    </button>
    
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
        <AIAssistant />
        <PageTransition :duration="800" @animation-end="triggerCardAnimations" />
        <SectionBackground>
          <!-- 检查是否是仪表盘根路径 -->
          <div v-if="route.path === '/dashboard'" class="dashboard-cards" :class="{ 'cards-animate': showCardAnimation }">
          <h2 class="dashboard-title">仪表盘</h2>
          
          <!-- 老师仪表盘 -->
          <div v-if="userRole === 'teacher'">
            <!-- 统计卡片 -->
            <div class="stats-cards">
              <div class="stat-card">
                <div class="stat-icon">
                  <el-icon><User /></el-icon>
                </div>
                <div class="stat-content">
                  <div class="stat-number">{{ teacherData.studentCount }}</div>
                  <div class="stat-label">班级人数</div>
                </div>
              </div>
              <div class="stat-card">
                <div class="stat-icon">
                  <el-icon><Timer /></el-icon>
                </div>
                <div class="stat-content">
                  <div class="stat-number">{{ teacherData.attendance.present }}</div>
                  <div class="stat-label">今日出勤</div>
                </div>
              </div>
              <div class="stat-card">
                <div class="stat-icon">
                  <el-icon><Ticket /></el-icon>
                </div>
                <div class="stat-content">
                  <div class="stat-number">{{ teacherData.todoList.length }}</div>
                  <div class="stat-label">待办事项</div>
                </div>
              </div>
              <div class="stat-card">
                <div class="stat-icon">
                  <el-icon><Top /></el-icon>
                </div>
                <div class="stat-content">
                  <div class="stat-number">0</div>
                  <div class="stat-label">平均分数</div>
                </div>
              </div>
            </div>
            
            <!-- 考勤统计 -->
            <div class="chart-card">
              <div class="card-header">
                <h3>考勤统计</h3>
              </div>
              <div class="card-body">
                <div class="attendance-stats">
                  <div class="attendance-stat">
                    <div class="attendance-stat-value">{{ teacherData.attendance.present }}</div>
                    <div class="attendance-stat-label">出勤</div>
                  </div>
                  <div class="attendance-stat">
                    <div class="attendance-stat-value">{{ teacherData.attendance.late }}</div>
                    <div class="attendance-stat-label">迟到</div>
                  </div>
                  <div class="attendance-stat">
                    <div class="attendance-stat-value">{{ teacherData.attendance.early }}</div>
                    <div class="attendance-stat-label">早退</div>
                  </div>
                  <div class="attendance-stat">
                    <div class="attendance-stat-value">{{ teacherData.attendance.absent }}</div>
                    <div class="attendance-stat-label">缺勤</div>
                  </div>
                </div>
                <div class="chart-placeholder">考勤图表区域</div>
              </div>
            </div>
            
            <!-- 待办事项 -->
            <div class="chart-card" style="margin-top: 20px;">
              <div class="card-header">
                <h3>待办事项</h3>
              </div>
              <div class="card-body">
                <div class="todo-list">
                  <div v-if="teacherData.todoList.length === 0" class="todo-item">
                    <div class="todo-content">
                      <div class="todo-title">暂无待办事项</div>
                    </div>
                  </div>
                  <div v-else v-for="todo in teacherData.todoList" :key="todo.id" class="todo-item">
                    <div class="todo-checkbox">
                      <el-checkbox></el-checkbox>
                    </div>
                    <div class="todo-content">
                      <div class="todo-title">{{ todo.title }}</div>
                      <div class="todo-due">{{ todo.due }}</div>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
          
          <!-- 学生仪表盘 -->
          <div v-else-if="userRole === 'student'">
            <!-- 统计卡片 -->
            <div class="stats-cards">
              <div class="stat-card">
                <div class="stat-icon">
                  <el-icon><Timer /></el-icon>
                </div>
                <div class="stat-content">
                  <div class="stat-number">{{ studentData.attendanceRate }}%</div>
                  <div class="stat-label">本周出勤</div>
                </div>
              </div>
              <div class="stat-card">
                <div class="stat-icon">
                  <el-icon><Top /></el-icon>
                </div>
                <div class="stat-content">
                  <div class="stat-number">{{ studentData.averageScore }}</div>
                  <div class="stat-label">平均分数</div>
                </div>
              </div>
              <div class="stat-card">
                <div class="stat-icon">
                  <el-icon><Ticket /></el-icon>
                </div>
                <div class="stat-content">
                  <div class="stat-number">{{ studentData.todoList.length }}</div>
                  <div class="stat-label">待办事项</div>
                </div>
              </div>
              <div class="stat-card">
                <div class="stat-icon">
                  <el-icon><Message /></el-icon>
                </div>
                <div class="stat-content">
                  <div class="stat-number">{{ studentData.messages.length }}</div>
                  <div class="stat-label">未读消息</div>
                </div>
              </div>
            </div>
            
            <!-- 待办事项 -->
            <div class="chart-card">
              <div class="card-header">
                <h3>待办事项</h3>
              </div>
              <div class="card-body">
                <div class="todo-list">
                  <div v-if="studentData.todoList.length === 0" class="todo-item">
                    <div class="todo-content">
                      <div class="todo-title">暂无待办事项</div>
                    </div>
                  </div>
                  <div v-else v-for="todo in studentData.todoList" :key="todo.id" class="todo-item">
                    <div class="todo-checkbox">
                      <el-checkbox></el-checkbox>
                    </div>
                    <div class="todo-content">
                      <div class="todo-title">{{ todo.title }}</div>
                      <div class="todo-due">{{ todo.due }}</div>
                    </div>
                  </div>
                </div>
              </div>
            </div>
            
            <!-- 消息通知 -->
            <div class="chart-card" style="margin-top: 20px;">
              <div class="card-header">
                <h3>消息通知</h3>
              </div>
              <div class="card-body">
                <div class="message-list">
                  <div v-if="studentData.messages.length === 0" class="message-item">
                    <div class="message-content">暂无消息通知</div>
                  </div>
                  <div v-else v-for="msg in studentData.messages" :key="msg.id" class="message-item">
                    <div class="message-header">
                      <div class="message-sender">{{ msg.sender }}</div>
                      <div class="message-time">{{ msg.time }}</div>
                    </div>
                    <div class="message-content">{{ msg.content }}</div>
                  </div>
                </div>
              </div>
            </div>
          </div>
          
          <!-- 管理员仪表盘 -->
          <div v-else>
            <!-- 统计卡片 -->
            <div class="stats-cards">
              <div class="stat-card">
                <div class="stat-icon">
                  <el-icon><User /></el-icon>
                </div>
                <div class="stat-content">
                  <div class="stat-number">{{ adminData.totalPersons }}</div>
                  <div class="stat-label">总人数</div>
                </div>
              </div>
              <div class="stat-card">
                <div class="stat-icon">
                  <el-icon><OfficeBuilding /></el-icon>
                </div>
                <div class="stat-content">
                  <div class="stat-number">{{ adminData.classCount }}</div>
                  <div class="stat-label">班级数</div>
                </div>
              </div>
              <div class="stat-card">
                <div class="stat-icon">
                  <el-icon><UserFilled /></el-icon>
                </div>
                <div class="stat-content">
                  <div class="stat-number">{{ adminData.teacherCount }}</div>
                  <div class="stat-label">教师数</div>
                </div>
              </div>
              <div class="stat-card">
                <div class="stat-icon">
                  <el-icon><Ticket /></el-icon>
                </div>
                <div class="stat-content">
                  <div class="stat-number">{{ adminData.todoCount }}</div>
                  <div class="stat-label">待办事项</div>
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
                  <div class="chart-placeholder">考勤图表区域</div>
                </div>
              </div>
              <div class="chart-card">
                <div class="card-header">
                  <h3>人员分布</h3>
                </div>
                <div class="card-body">
                  <div class="chart-placeholder">人员分布图表区域</div>
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
                  <div v-if="adminData.activities.length === 0" class="activity-item">
                    <div class="activity-icon"></div>
                    <div class="activity-content">
                      <div class="activity-title">暂无动态</div>
                    </div>
                  </div>
                  <div v-for="activity in adminData.activities" :key="activity.id" class="activity-item">
                    <div class="activity-icon" :class="activity.type"></div>
                    <div class="activity-content">
                      <div class="activity-title">{{ activity.title }}</div>
                      <div class="activity-desc">{{ activity.description }}</div>
                      <div class="activity-time">{{ new Date(activity.created_at).toLocaleString() }}</div>
                    </div>
                  </div>
                </div>
              </div>
            </div>
            
            <!-- 公告通知 -->
            <div class="chart-card" style="margin-top: 20px;">
              <div class="card-header">
                <h3>公告通知</h3>
              </div>
              <div class="card-body">
                <div class="notice-list">
                  <div v-if="adminData.notices.length === 0" class="notice-item">
                    <div class="notice-content">暂无公告</div>
                  </div>
                  <div v-for="notice in adminData.notices" :key="notice.id" class="notice-item" :class="{ 'is-important': notice.is_important }">
                    <div class="notice-title">
                      <span v-if="notice.is_important" class="important-tag">重要</span>
                      {{ notice.title }}
                    </div>
                    <div class="notice-content">{{ notice.content }}</div>
                    <div class="notice-meta">
                      <span>{{ notice.author_name }}</span>
                      <span>{{ new Date(notice.created_at).toLocaleDateString() }}</span>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
        
          <!-- 其他路由内容 -->
          <router-view v-else />
        </SectionBackground>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed, nextTick, watch } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { 
  House, User, UserFilled, Timer, GoodsFilled, Message, Bell, Setting, 
  Suitcase, OfficeBuilding, DataAnalysis, Operation, Money, TrendCharts, 
  Collection, Top, Reading, Star, Check, Flag, Edit, Calendar, 
  Tickets, Trophy, Lock, Grid, Document, Ticket, ChatLineRound
} from '@element-plus/icons-vue'
import { useAuthStore } from '../store/auth'
import { menuItems, menuGroups } from '../config/menu'
import type { MenuItem } from '../config/types'
import { personApi } from '../api/person'
import { attendanceApi } from '../api/attendance'
import { classApi } from '../api/class'
import { noticeApi } from '../api/notice'
import { scoreApi } from '../api/score'
import SectionBackground from '../components/SectionBackground.vue'
import PageTransition from '../components/PageTransition.vue'
import AIAssistant from '../components/AIAssistant.vue'
import '@styles/dashboard.css'
import { useMobile } from '../composables/useMobile'

// 移动端适配
const { isMobile, loadMobileStyle } = useMobile()

// 动态加载移动端样式
onMounted(async () => {
  if (isMobile.value) {
    await loadMobileStyle('dashboard-mobile')
  }
})

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
  Ticket,
  ChatLineRound
}

const router = useRouter()
const route = useRoute()
const authStore = useAuthStore()
const userMenuVisible = ref(false)

const sidebarRef = ref<HTMLElement | null>(null)
const dashboardItemRef = ref<HTMLElement | null>(null)
const menuItemRefs = ref<Record<string, HTMLElement>>({})
const activeBgStyle = ref<{ transform: string } | null>(null)
const showCardAnimation = ref(false)

const setMenuItemRef = (el: any, id: string) => {
  if (el) {
    menuItemRefs.value[id] = el
  }
}

const animateCardsRecursively = (
  elements: NodeListOf<Element> | Element[],
  index: number = 0,
  delay: number = 100
) => {
  if (index >= elements.length) return
  
  const element = elements[index] as HTMLElement
  element.style.transitionDelay = `${index * delay}ms`
  element.classList.add('card-animated')
  
  requestAnimationFrame(() => {
    animateCardsRecursively(elements, index + 1, delay)
  })
}

const triggerCardAnimations = async () => {
  showCardAnimation.value = false
  
  await nextTick()
  
  const contentEl = document.querySelector('.content')
  if (!contentEl) return
  
  const allCards = contentEl.querySelectorAll('.stat-card, .chart-card, .activity-card')
  allCards.forEach(card => {
    card.classList.remove('card-animated')
    ;(card as HTMLElement).style.transitionDelay = ''
  })
  
  await nextTick()
  
  showCardAnimation.value = true
  animateCardsRecursively(Array.from(allCards))
}

const updateActiveBgPosition = async () => {
  await nextTick()
  
  let activeElement: HTMLElement | null = null
  
  if (isDashboardActive.value && dashboardItemRef.value) {
    activeElement = dashboardItemRef.value
  } else {
    const activeItem = filteredMenuItems.value.find(item => isMenuItemActive(item))
    if (activeItem && menuItemRefs.value[activeItem.id]) {
      activeElement = menuItemRefs.value[activeItem.id]
    }
  }
  
  if (activeElement && sidebarRef.value) {
    const sidebarRect = sidebarRef.value.getBoundingClientRect()
    const elementRect = activeElement.getBoundingClientRect()
    const offsetTop = elementRect.top - sidebarRect.top + sidebarRef.value.scrollTop
    
    activeBgStyle.value = {
      transform: `translateY(${offsetTop}px)`
    }
  }
}

watch(() => route.path, () => {
  updateActiveBgPosition()
}, { immediate: true })

// 类型定义
interface TodoItem {
  id: string
  title: string
  due: string
}

interface MessageItem {
  id: string
  sender: string
  time: string
  content: string
}

interface AttendanceData {
  total: number
  present: number
  late: number
  absent: number
  early: number
}

interface TeacherData {
  classCount: number
  studentCount: number
  attendance: AttendanceData
  todoList: TodoItem[]
}

interface StudentData {
  attendanceRate: number
  averageScore: number
  todoList: TodoItem[]
  messages: MessageItem[]
}

interface AdminData {
  totalPersons: number
  classCount: number
  teacherCount: number
  todoCount: number
  notices: NoticeItem[]
  activities: ActivityItem[]
}

interface NoticeItem {
  id: string
  title: string
  content: string
  author_name: string
  is_important: boolean
  created_at: string
}

interface ActivityItem {
  id: string
  type: string
  title: string
  description: string
  created_at: string
}

// 数据状态
const loading = ref(false)
const teacherData = ref<TeacherData>({
  classCount: 0,
  studentCount: 0,
  attendance: {
    total: 0,
    present: 0,
    late: 0,
    absent: 0,
    early: 0
  },
  todoList: []
})

const studentData = ref<StudentData>({
  attendanceRate: 0,
  averageScore: 0,
  todoList: [],
  messages: []
})

const adminData = ref<AdminData>({
  totalPersons: 0,
  classCount: 0,
  teacherCount: 0,
  todoCount: 0,
  notices: [],
  activities: []
})

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

// 获取老师数据
const fetchTeacherData = async () => {
  console.log('[DashboardView] fetchTeacherData called')
  console.log('[DashboardView] Teacher user ID:', authStore.user?.id)
  try {
    loading.value = true
    
    // 获取老师关联的班级
    console.log('[DashboardView] Calling personApi.getTeacherClasses with teacher ID:', authStore.user?.id)
    let classesResponse
    try {
      classesResponse = await personApi.getTeacherClasses(authStore.user?.id || '')
      console.log('[DashboardView] Teacher classes response:', classesResponse)
      console.log('[DashboardView] Teacher classes data:', classesResponse.data)
    } catch (error) {
      console.error('[DashboardView] 获取老师班级失败:', error)
      console.error('[DashboardView] Error details:', error instanceof Error ? error.message : String(error))
      classesResponse = { data: [] }
    }
    
    const classIds = classesResponse.data.map((cls: any) => cls.id)
    console.log('[DashboardView] Teacher class IDs:', classIds)
    teacherData.value.classCount = classIds.length
    
    if (classIds.length === 0) {
      console.warn('[DashboardView] 老师没有关联的班级，考勤数据将为空')
    }
    
    // 获取班级学生数量
    let totalStudents = 0
    if (classIds.length > 0) {
      const studentResults = await Promise.allSettled(
        classIds.map(async (classId: string) => {
          console.log('[DashboardView] 获取班级学生, classId:', classId)
          const studentsResponse = await personApi.list({
            page: 1,
            limit: 1000,
            type: 'student',
            class_id: classId
          })
          console.log('[DashboardView] 班级学生响应, classId:', classId, 'total:', studentsResponse.data.total)
          return studentsResponse.data.total
        })
      )

      totalStudents = studentResults.reduce((sum, result, index) => {
        if (result.status === 'fulfilled') {
          return sum + result.value
        }

        console.error('[DashboardView] 获取班级学生失败, classId:', classIds[index], 'error:', result.reason)
        return sum
      }, 0)
    }
    teacherData.value.studentCount = totalStudents
    console.log('[DashboardView] 总学生数:', totalStudents)
    
    // 获取最近7天考勤（只获取教师所管理班级的学生考勤）
    const today = new Date()
    const sevenDaysAgo = new Date(today)
    sevenDaysAgo.setDate(today.getDate() - 7)
    sevenDaysAgo.setHours(0, 0, 0, 0)
    
    console.log('[DashboardView] 查询最近7天考勤，日期范围:', sevenDaysAgo.toISOString(), '至', today.toISOString())
    
    let allAttendanceData: any[] = []
    
    if (classIds.length > 0) {
      const attendanceResults = await Promise.allSettled(
        classIds.map(async (classId: string) => {
          console.log('[DashboardView] Fetching attendance for class ID:', classId)
          const attendanceResponse = await attendanceApi.list({
            page: 1,
            limit: 1000,
            class_id: classId
          })
          console.log('[DashboardView] Attendance response for class', classId, ':', attendanceResponse.data)
          console.log('[DashboardView] Attendance items count:', attendanceResponse.data.items.length)

          const recentAttendance = attendanceResponse.data.items.filter((item: any) => {
            const itemDate = new Date(item.date)
            return itemDate >= sevenDaysAgo
          })
          console.log('[DashboardView] Recent attendance (last 7 days) for class', classId, ':', recentAttendance.length)
          return recentAttendance
        })
      )

      allAttendanceData = attendanceResults.flatMap((result, index) => {
        if (result.status === 'fulfilled') {
          return result.value
        }

        console.error('[DashboardView] 获取考勤数据失败, classId:', classIds[index], 'error:', result.reason)
        console.error('[DashboardView] Error details:', result.reason instanceof Error ? result.reason.message : String(result.reason))
        return []
      })
    } else {
      console.warn('[DashboardView] 没有班级ID，跳过考勤查询')
    }
    console.log('[DashboardView] Total attendance data (last 7 days):', allAttendanceData)
    console.log('[DashboardView] Total attendance count (last 7 days):', allAttendanceData.length)
    
    teacherData.value.attendance.total = allAttendanceData.length
    teacherData.value.attendance.present = allAttendanceData.filter((item: any) => item.status === 'present').length
    teacherData.value.attendance.late = allAttendanceData.filter((item: any) => item.status === 'late').length
    teacherData.value.attendance.absent = allAttendanceData.filter((item: any) => item.status === 'absent').length
    teacherData.value.attendance.early = allAttendanceData.filter((item: any) => item.status === 'early_leave').length
    
    console.log('[DashboardView] 考勤统计结果:')
    console.log('[DashboardView] 总考勤记录:', teacherData.value.attendance.total)
    console.log('[DashboardView] 正常出勤:', teacherData.value.attendance.present)
    console.log('[DashboardView] 迟到:', teacherData.value.attendance.late)
    console.log('[DashboardView] 缺勤:', teacherData.value.attendance.absent)
    console.log('[DashboardView] 早退:', teacherData.value.attendance.early)
    
  } catch (error) {
    console.error('[DashboardView] 获取老师数据失败:', error)
    console.error('[DashboardView] Error details:', error instanceof Error ? error.message : String(error))
  } finally {
    console.log('[DashboardView] fetchTeacherData completed, loading set to false')
    loading.value = false
  }
}

// 获取学生数据
const fetchStudentData = async () => {
  console.log('[DashboardView] fetchStudentData called')
  console.log('[DashboardView] Student user ID:', authStore.user?.id)
  try {
    loading.value = true
    
    // 获取学生本周考勤
    const studentId = authStore.user?.id
    console.log('[DashboardView] Student ID for attendance query:', studentId)
    
    if (!studentId) {
      console.error('[DashboardView] 学生ID为空，无法获取考勤数据')
      studentData.value.attendanceRate = 0
      return
    }
    
    const today = new Date()
    const weekStart = new Date(today)
    weekStart.setDate(today.getDate() - today.getDay() + 1)
    weekStart.setHours(0, 0, 0, 0)
    const weekEnd = new Date(today)
    weekEnd.setDate(today.getDate() + (7 - today.getDay()))
    weekEnd.setHours(23, 59, 59, 999)
    
    console.log('[DashboardView] 查询学生考勤, studentId:', studentId, '本周范围:', weekStart.toISOString(), '至', weekEnd.toISOString())
    
    try {
      // 获取学生考勤记录（不指定日期，获取所有记录再过滤）
      const attendanceResponse = await attendanceApi.list({
        page: 1,
        limit: 1000,
        person_id: studentId
      })
      console.log('[DashboardView] 学生考勤响应:', attendanceResponse.data)
      console.log('[DashboardView] 考勤记录总数:', attendanceResponse.data.items.length)
      
      // 过滤出本周的记录
      const thisWeekData = attendanceResponse.data.items.filter((item: any) => {
        const itemDate = new Date(item.date)
        return itemDate >= weekStart && itemDate <= weekEnd
      })
      
      console.log('[DashboardView] 本周考勤记录:', thisWeekData.length)
      console.log('[DashboardView] 本周考勤详情:', thisWeekData)
      
      const totalDays = thisWeekData.length
      const presentDays = thisWeekData.filter((item: any) => item.status === 'present').length
      
      console.log('[DashboardView] 本周总天数:', totalDays)
      console.log('[DashboardView] 正常出勤天数:', presentDays)
      
      studentData.value.attendanceRate = totalDays > 0 ? Math.round((presentDays / totalDays) * 100) : 0
      console.log('[DashboardView] 出勤率:', studentData.value.attendanceRate, '%')
      
    } catch (error) {
      console.error('[DashboardView] 获取学生考勤失败:', error)
      console.error('[DashboardView] Error details:', error instanceof Error ? error.message : String(error))
      studentData.value.attendanceRate = 0
    }
    
    // 获取学生消息通知（学校公告和班级公告）
    try {
      console.log('[DashboardView] 获取学生消息通知')
      
      // 获取学生详细信息（包含班级ID）
      const studentInfoResponse = await personApi.get(studentId)
      console.log('[DashboardView] 学生详细信息:', studentInfoResponse.data)
      
      const studentInfo = studentInfoResponse.data as any
      const classId = studentInfo.class_id
      console.log('[DashboardView] 学生班级ID:', classId)
      
      const allMessages: MessageItem[] = []
      
      // 获取学校公告（公开公告）
      console.log('[DashboardView] 获取学校公告')
      const schoolNoticesResponse = await noticeApi.list({
        page: 1,
        limit: 10,
        target_type: 'school'
      })
      console.log('[DashboardView] 学校公告响应:', schoolNoticesResponse.data)
      
      schoolNoticesResponse.data.items.forEach((notice: any) => {
        allMessages.push({
          id: notice.id,
          sender: notice.author_name || '学校',
          time: notice.created_at,
          content: notice.title + ': ' + notice.content.substring(0, 50) + (notice.content.length > 50 ? '...' : '')
        })
      })
      
      // 获取班级公告（如果学生有关联班级）
      if (classId) {
        console.log('[DashboardView] 获取班级公告, classId:', classId)
        const classNoticesResponse = await noticeApi.list({
          page: 1,
          limit: 10,
          target_type: 'class',
          target_id: classId
        })
        console.log('[DashboardView] 班级公告响应:', classNoticesResponse.data)
        
        classNoticesResponse.data.items.forEach((notice: any) => {
          allMessages.push({
            id: notice.id,
            sender: notice.author_name || '班级',
            time: notice.created_at,
            content: notice.title + ': ' + notice.content.substring(0, 50) + (notice.content.length > 50 ? '...' : '')
          })
        })
      } else {
        console.warn('[DashboardView] 学生没有关联班级，跳过班级公告获取')
      }
      
      // 按时间倒序排序
      allMessages.sort((a, b) => new Date(b.time).getTime() - new Date(a.time).getTime())
      
      // 只保留最新的5条消息
      studentData.value.messages = allMessages.slice(0, 5)
      console.log('[DashboardView] 学生消息通知:', studentData.value.messages)
      
    } catch (error) {
      console.error('[DashboardView] 获取学生消息通知失败:', error)
      console.error('[DashboardView] Error details:', error instanceof Error ? error.message : String(error))
      studentData.value.messages = []
    }
    

  } catch (error) {
    console.error('[DashboardView] 获取学生数据失败:', error)
    console.error('[DashboardView] Error details:', error instanceof Error ? error.message : String(error))
  } finally {
    console.log('[DashboardView] fetchStudentData completed, loading set to false')
    loading.value = false
  }
}

// 获取管理员数据
const fetchAdminData = async () => {
  console.log('[DashboardView] fetchAdminData called')
  console.log('[DashboardView] Admin user ID:', authStore.user?.id)
  try {
    loading.value = true
    
    // 获取总人数
    const totalPersonsResponse = await personApi.list({
      page: 1,
      limit: 1000
    })
    adminData.value.totalPersons = totalPersonsResponse.data.total
    
    // 获取教师数量
    const teachersResponse = await personApi.list({
      page: 1,
      limit: 1000,
      type: 'teacher'
    })
    adminData.value.teacherCount = teachersResponse.data.total
    
    // 获取班级数量
    const classesResponse = await classApi.list({
      page: 1,
      limit: 1000
    })
    adminData.value.classCount = classesResponse.data.total
    
    // 获取公告列表
    try {
      console.log('[DashboardView] Fetching notices')
      const noticesResponse = await noticeApi.list({
        page: 1,
        limit: 5
      })
      console.log('[DashboardView] Notices response:', noticesResponse.data)
      adminData.value.notices = noticesResponse.data.items.map((item: any) => ({
        id: item.id,
        title: item.title,
        content: item.content,
        author_name: item.author_name,
        is_important: item.is_important,
        created_at: item.created_at
      }))
      console.log('[DashboardView] Processed notices:', adminData.value.notices)
    } catch (error) {
      console.error('[DashboardView] 获取公告失败:', error)
      adminData.value.notices = []
    }
    
    // 获取最近动态（基于考勤和评分记录）
    try {
      console.log('[DashboardView] Fetching activities (attendance and scores)')
      const [attendanceRes, scoresRes] = await Promise.all([
        attendanceApi.list({ page: 1, limit: 5 }),
        scoreApi.list({ page: 1, limit: 5 })
      ])
      console.log('[DashboardView] Attendance activities response:', attendanceRes.data)
      console.log('[DashboardView] Score activities response:', scoresRes.data)
      
      const activities: ActivityItem[] = []
      
      attendanceRes.data.items.forEach((item: any) => {
        activities.push({
          id: item.id,
          type: 'attendance',
          title: `${item.person_name} 考勤记录`,
          description: `状态: ${item.status === 'present' ? '正常' : item.status === 'late' ? '迟到' : item.status === 'absent' ? '缺勤' : item.status === 'early_leave' ? '早退' : '请假'}`,
          created_at: item.created_at
        })
      })
      
      scoresRes.data.items.forEach((item: any) => {
        activities.push({
          id: item.id,
          type: 'score',
          title: `${item.person_name} 评分记录`,
          description: `分数: ${item.value}`,
          created_at: item.created_at
        })
      })
      
      activities.sort((a, b) => new Date(b.created_at).getTime() - new Date(a.created_at).getTime())
      adminData.value.activities = activities.slice(0, 10)
    } catch (error) {
      console.error('获取动态失败:', error)
      adminData.value.activities = []
    }
    
  } catch (error) {
    console.error('[DashboardView] 获取管理员数据失败:', error)
    console.error('[DashboardView] Error details:', error instanceof Error ? error.message : String(error))
  } finally {
    console.log('[DashboardView] fetchAdminData completed, loading set to false')
    loading.value = false
  }
}

// 加载数据
const loadDashboardData = async () => {
  console.log('[DashboardView] loadDashboardData called, userRole:', userRole.value)
  try {
    switch (userRole.value) {
      case 'teacher':
        console.log('[DashboardView] Fetching teacher data')
        await fetchTeacherData()
        break
      case 'student':
        console.log('[DashboardView] Fetching student data')
        await fetchStudentData()
        break
      default:
        console.log('[DashboardView] Fetching admin data')
        await fetchAdminData()
        break
    }
    console.log('[DashboardView] loadDashboardData completed')
  } catch (error) {
    console.error('[DashboardView] loadDashboardData failed:', error)
  }
}

const toggleUserMenu = () => {
  userMenuVisible.value = !userMenuVisible.value
}

const handleLogout = () => {
  authStore.clearAuth()
  router.push('/login')
}

// 回到顶部按钮显示状态
const showScrollToTop = ref(false)

// 滚动到顶部
const scrollToTop = () => {
  const sidebar = document.querySelector('.sidebar')
  if (sidebar) {
    sidebar.scrollTo({ top: 0, behavior: 'smooth' })
  }
}

// 处理侧边栏滚动事件
const handleSidebarScroll = () => {
  const sidebar = document.querySelector('.sidebar')
  if (sidebar) {
    // 当滚动距离超过50px时显示按钮
    showScrollToTop.value = sidebar.scrollTop > 50
  }
}

onMounted(() => {
  console.log('[DashboardView] onMounted called')
  console.log('[DashboardView] authStore.isAuthenticated:', authStore.isAuthenticated)
  console.log('[DashboardView] userRole:', userRole.value)
  console.log('[DashboardView] authStore.user:', authStore.user)
  
  // 检查是否有token
  if (!authStore.isAuthenticated) {
    console.log('[DashboardView] User not authenticated, redirecting to login')
    router.push('/login')
    return
  }
  
  console.log('[DashboardView] Loading dashboard data')
  // 加载仪表盘数据
  loadDashboardData()
  
  // 首次加载触发卡片动画
  setTimeout(() => {
    showCardAnimation.value = true
  }, 100)
  
  // 点击外部关闭用户菜单
  document.addEventListener('click', (e) => {
    const target = e.target as HTMLElement
    if (!target.closest('.user-avatar') && !target.closest('.user-menu')) {
      userMenuVisible.value = false
    }
  })
  
  // 监听侧边栏滚动事件
  const sidebar = document.querySelector('.sidebar')
  if (sidebar) {
    sidebar.addEventListener('scroll', handleSidebarScroll)
  }
})
</script>
