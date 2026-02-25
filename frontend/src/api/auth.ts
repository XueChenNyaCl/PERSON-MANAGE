import api from './index'

export interface LoginRequest {
  username: string
  password: string
  remember_me?: boolean
}

export interface LoginResponse {
  token: string
  user: UserInfo
  permissions: string[]
  expires_in: number
}

export interface UserInfo {
  id: string
  username: string
  role: string
  name: string
  email: string
}

export const authApi = {
  login: (data: LoginRequest) => {
    return api.post<LoginResponse>('/auth/login', data)
  },
  
  logout: () => {
    return api.post('/auth/logout')
  },
  
  getCurrentUser: () => {
    return api.get<UserInfo>('/user/me')
  },
  
  refreshToken: () => {
    return api.post('/auth/refresh')
  }
}