import { createRouter, createWebHistory } from 'vue-router'
import LoginView from '../views/LoginView.vue'
import HomeView from '../views/HomeView.vue'
import DashboardView from '../views/DashboardView.vue'
import { useAuthStore } from '../store/auth'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      name: 'home',
      component: HomeView
    },
    {
      path: '/login',
      name: 'login',
      component: LoginView
    },
    {
      path: '/dashboard',
      name: 'dashboard',
      component: DashboardView,
      children: [
        {
          path: '',
          name: 'dashboard-default',
          component: { render: () => null } // 空组件，这样仪表盘内容会显示
        },
        {
          path: 'person',
          name: 'person',
          component: () => import('../views/PersonView.vue')
        },
        {
          path: 'class',
          name: 'class',
          component: () => import('../views/ClassView.vue')
        },
        {
          path: 'department',
          name: 'department',
          component: () => import('../views/DepartmentView.vue')
        },
        {
          path: 'class/manage',
          name: 'class-manage',
          component: () => import('../views/ClassManageView.vue')
        },
        {
          path: 'attendance',
          name: 'attendance',
          component: () => import('../views/AttendanceView.vue')
        },
        {
          path: 'score',
          name: 'score',
          component: () => import('../views/ScoreView.vue')
        },
        {
          path: 'notice',
          name: 'notice',
          component: () => import('../views/NoticeView.vue')
        }
      ]
    },
    // 重定向其他路径到dashboard下对应的子路由
    {
      path: '/person',
      redirect: '/dashboard/person'
    },
    {
      path: '/class',
      redirect: '/dashboard/class'
    },
    {
      path: '/department',
      redirect: '/dashboard/department'
    },
    {
      path: '/attendance',
      redirect: '/dashboard/attendance'
    },
    {
      path: '/score',
      redirect: '/dashboard/score'
    },
    {
      path: '/notice',
      redirect: '/dashboard/notice'
    }
  ]
})

// 路由守卫
router.beforeEach((to, from, next) => {
  const authStore = useAuthStore()
  
  // 检查是否需要认证
  const publicRoutes = ['/login', '/']
  const isPublicRoute = publicRoutes.includes(to.path)
  
  if (!isPublicRoute && !authStore.isAuthenticated) {
    // 重定向到登录页
    next('/login')
    return
  }
  
  // 如果已登录且访问登录页，重定向到dashboard
  if (to.path === '/login' && authStore.isAuthenticated) {
    next('/dashboard')
    return
  }
  
  next()
})

export default router
