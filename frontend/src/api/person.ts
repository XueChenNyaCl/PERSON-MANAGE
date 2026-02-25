import api from './index'

// 人员类型定义
export interface PersonBase {
  id: string
  name: string
  gender: number
  birthday?: string
  phone?: string
  email?: string
  type: 'student' | 'teacher' | 'parent'
}

// 老师班级关联创建结构
export interface TeacherClassCreate {
  class_id: string
  is_main_teacher: boolean
}

// 老师班级关联响应结构
export interface TeacherClassResponse {
  id: string
  name: string
  grade: number
  academic_year: string
  is_main_teacher: boolean
}

export interface StudentResponse extends PersonBase {
  type: 'student'
  student_no: string
  class_id?: string
  class_name?: string
  enrollment_date?: string
  status: string
}

export interface TeacherResponse extends PersonBase {
  type: 'teacher'
  employee_no: string
  department_id?: string
  department_name?: string
  title?: string
  hire_date?: string
  // 老师关联的多个班级
  classes?: TeacherClassCreate[]
}

export interface ParentResponse extends PersonBase {
  type: 'parent'
  wechat_openid?: string
  occupation?: string
}

export type PersonResponse = StudentResponse | TeacherResponse | ParentResponse

export interface PersonCreate {
  name: string
  gender: number
  birthday?: string
  phone?: string
  email?: string
  type_: string
  student_no?: string
  class_id?: string
  enrollment_date?: string
  employee_no?: string
  department_id?: string
  title?: string
  hire_date?: string
  wechat_openid?: string
  occupation?: string
  // 老师关联的多个班级
  classes?: TeacherClassCreate[]
}

export interface PersonUpdate {
  name?: string
  gender?: number
  birthday?: string
  phone?: string
  email?: string
  student_no?: string
  class_id?: string
  enrollment_date?: string
  employee_no?: string
  department_id?: string
  title?: string
  hire_date?: string
  wechat_openid?: string
  occupation?: string
  // 老师关联的多个班级
  classes?: TeacherClassCreate[]
}

export interface PersonQuery {
  page: number
  limit: number
  type?: string
  search?: string
  class_id?: string
  department_id?: string
}

export interface ListResponse<T> {
  items: T[]
  total: number
  page: number
  limit: number
}

// 人员管理API
export const personApi = {
  // 获取人员列表
  list: (params: PersonQuery) => {
    return api.get<ListResponse<PersonResponse>>('/persons', { params })
  },
  
  // 创建人员
  create: (data: PersonCreate) => {
    return api.post<PersonResponse>('/persons', data)
  },
  
  // 获取单个人员
  get: (id: string) => {
    return api.get<PersonResponse>(`/persons/${id}`)
  },
  
  // 更新人员
  update: (id: string, data: PersonUpdate) => {
    return api.put<PersonResponse>(`/persons/${id}`, data)
  },
  
  // 删除人员
  delete: (id: string) => {
    return api.delete(`/persons/${id}`)
  },
  
  // 获取老师关联的班级
  getTeacherClasses: (teacherId: string) => {
    return api.get<TeacherClassResponse[]>('/permission/teacher/classes', { params: { teacher_id: teacherId } })
  }
}
