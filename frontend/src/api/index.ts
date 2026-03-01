import axios from 'axios'
import { aiApi } from './ai'

const api = axios.create({
  baseURL: '/api',
  timeout: 10000,
})

// 导出所有 API 模块
export const apiModules = {
  ai: aiApi
}

// 请求拦截器：添加JWT令牌
api.interceptors.request.use(
  (config) => {
    // 登录和注册请求不需要添加token
    const publicUrls = ['/auth/login', '/auth/register']
    const isPublicUrl = publicUrls.some(url => config.url?.includes(url))
    
    if (!isPublicUrl) {
      const token = localStorage.getItem('token')
      if (token) {
        config.headers.Authorization = `Bearer ${token}`
      }
    }
    return config
  },
  (error) => {
    return Promise.reject(error)
  }
)

// 响应拦截器：处理401错误
api.interceptors.response.use(
  (response) => response,
  async (error) => {
    const originalRequest = error.config
    
    // 401错误且不是登录请求
    if (error.response?.status === 401 && !originalRequest._retry && originalRequest.url !== '/auth/login') {
      originalRequest._retry = true
      
      try {
        // 尝试刷新令牌
        const refreshToken = localStorage.getItem('refresh_token')
        if (refreshToken) {
          const response = await api.post('/auth/refresh', { refresh_token: refreshToken })
          const newToken = response.data.token
          localStorage.setItem('token', newToken)
          
          // 重试原始请求
          originalRequest.headers.Authorization = `Bearer ${newToken}`
          return api(originalRequest)
        }
      } catch (refreshError) {
        // 刷新失败，跳转到登录页
        localStorage.clear()
        window.location.href = '/login'
        return Promise.reject(refreshError)
      }
    }
    
    return Promise.reject(error)
  }
)

export default api