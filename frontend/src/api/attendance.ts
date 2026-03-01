import api from './index'

// 考勤查询参数
export interface AttendanceQuery {
  page: number
  limit: number
  name?: string
  date?: string
  status?: string
}

// 考勤创建参数
export interface AttendanceCreate {
  person_id: string
  date: string
  status: 'present' | 'late' | 'absent' | 'early_leave' | 'excused'
  time?: string
  remark?: string
}

// 考勤更新参数
export interface AttendanceUpdate {
  status?: 'present' | 'late' | 'absent' | 'early_leave' | 'excused'
  time?: string
  remark?: string
}

// 考勤响应数据
export interface AttendanceResponse {
  id: string
  person_id: string
  person_name: string
  date: string
  status: 'present' | 'late' | 'absent' | 'early_leave' | 'excused'
  time?: string
  remark?: string
  created_at: string
}

// 列表响应
export interface ListResponse<T> {
  items: T[]
  total: number
}

// 考勤API
export const attendanceApi = {
  list: (params: AttendanceQuery) => {
    return api.get<ListResponse<AttendanceResponse>>('/attendances', { params })
  },
  create: (data: AttendanceCreate) => {
    return api.post<AttendanceResponse>('/attendances', data)
  },
  get: (id: string) => {
    return api.get<AttendanceResponse>(`/attendances/${id}`)
  },
  update: (id: string, data: AttendanceUpdate) => {
    return api.put<AttendanceResponse>(`/attendances/${id}`, data)
  },
  delete: (id: string) => {
    return api.delete(`/attendances/${id}`)
  }
}
