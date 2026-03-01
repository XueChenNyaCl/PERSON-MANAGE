<template>
  <div class="dashboard-container">
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
                  <div class="activity-item">
                    <div class="activity-icon"></div>
                    <div class="activity-content">
                      <div class="activity-title">系统启动</div>
                      <div class="activity-time">{{ new Date().toLocaleString() }}</div>
                    </div>
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
  Tickets, Trophy, Lock, Grid, Document, Ticket, ChatLineRound 
} from '@element-plus/icons-vue'
import { useAuthStore } from '../store/auth'
import { menuItems, menuGroups } from '../config/menu'
import type { MenuItem } from '../config/types'
import { personApi } from '../api/person'
import { attendanceApi } from '../api/attendance'
import { classApi } from '../api/class'
import '../styles/dashboard.css'

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
  todoCount: 0
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
  try {
    loading.value = true
    
    // 获取老师关联的班级
    const classesResponse = await personApi.getTeacherClasses(authStore.user?.id || '')
    const classIds = classesResponse.data.map((cls: any) => cls.id)
    teacherData.value.classCount = classIds.length
    
    // 获取班级学生数量
    let totalStudents = 0
    for (const classId of classIds) {
      const studentsResponse = await personApi.list({
        page: 1,
        limit: 1000,
        type: 'student',
        class_id: classId
      })
      totalStudents += studentsResponse.data.total
    }
    teacherData.value.studentCount = totalStudents
    
    // 获取今日考勤
    const today = new Date().toISOString().split('T')[0]
    const attendanceResponse = await attendanceApi.list({
      page: 1,
      limit: 1000,
      date: today
    })
    
    const attendanceData = attendanceResponse.data.items
    teacherData.value.attendance.total = attendanceData.length
    teacherData.value.attendance.present = attendanceData.filter((item: any) => item.status === '正常').length
    teacherData.value.attendance.late = attendanceData.filter((item: any) => item.status === '迟到').length
    teacherData.value.attendance.absent = attendanceData.filter((item: any) => item.status === '缺勤').length
    teacherData.value.attendance.early = attendanceData.filter((item: any) => item.status === '早退').length
    
  } catch (error) {
    console.error('获取老师数据失败:', error)
  } finally {
    loading.value = false
  }
}

// 获取学生数据
const fetchStudentData = async () => {
  try {
    loading.value = true
    
    // 获取本周考勤
    const today = new Date()
    const weekStart = new Date(today)
    weekStart.setDate(today.getDate() - today.getDay() + 1)
    const weekEnd = new Date(today)
    weekEnd.setDate(today.getDate() + (7 - today.getDay()))
    
    const attendanceResponse = await attendanceApi.list({
      page: 1,
      limit: 1000,
      date: weekStart.toISOString().split('T')[0]
    })
    
    const attendanceData = attendanceResponse.data.items
    const totalDays = attendanceData.length
    const presentDays = attendanceData.filter((item: any) => item.status === '正常').length
    studentData.value.attendanceRate = totalDays > 0 ? Math.round((presentDays / totalDays) * 100) : 0
    
    // 模拟消息数据
    studentData.value.messages = [
      {
        id: '1',
        sender: '张老师',
        time: '今天 10:30',
        content: '数学作业已经发布，请及时完成。'
      },
      {
        id: '2',
        sender: '系统',
        time: '今天 09:00',
        content: '明天将进行英语单元测试，请做好准备。'
      },
      {
        id: '3',
        sender: '李同学',
        time: '昨天 16:45',
        content: '关于小组项目的讨论，我们明天课间碰个面。'
      }
    ]
    
  } catch (error) {
    console.error('获取学生数据失败:', error)
  } finally {
    loading.value = false
  }
}

// 获取管理员数据
const fetchAdminData = async () => {
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
    
  } catch (error) {
    console.error('获取管理员数据失败:', error)
  } finally {
    loading.value = false
  }
}

// 加载数据
const loadDashboardData = async () => {
  switch (userRole.value) {
    case 'teacher':
      await fetchTeacherData()
      break
    case 'student':
      await fetchStudentData()
      break
    default:
      await fetchAdminData()
      break
  }
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
    return
  }
  
  // 加载仪表盘数据
  loadDashboardData()
  
  // 点击外部关闭用户菜单
  document.addEventListener('click', (e) => {
    const target = e.target as HTMLElement
    if (!target.closest('.user-avatar') && !target.closest('.user-menu')) {
      userMenuVisible.value = false
    }
  })
})
</script>
