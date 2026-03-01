import api from './index'

export interface Group {
  id: string
  class_id: string
  class_name?: string
  name: string
  description?: string
  score: number
  member_count: number
  created_at: string
  updated_at: string
}

export interface GroupCreate {
  class_id: string
  name: string
  description?: string
}

export interface GroupUpdate {
  name?: string
  description?: string
}

export interface GroupMemberAdd {
  person_id: string
}

export interface GroupScoreChange {
  score_change: number
  reason: string
}

export interface GroupScoreRecord {
  id: string
  group_id: string
  score_change: number
  reason: string
  created_by: string
  created_at: string
}

export interface GroupMember {
  id: string
  name: string
  gender: number
  birthday?: string
  phone?: string
  email?: string
  student_no: string
  class_id?: string
  class_name?: string
  enrollment_date?: string
  status: string
}

export const groupApi = {
  // 获取所有小组列表
  list: async (): Promise<Group[]> => {
    const response = await api.get('/groups')
    return response.data
  },

  // 获取班级的小组列表
  getGroupsByClass: async (classId: string): Promise<Group[]> => {
    const response = await api.get(`/groups/class/${classId}`)
    return response.data
  },

  // 创建小组
  createGroup: async (data: GroupCreate): Promise<Group> => {
    const response = await api.post('/groups', data)
    return response.data
  },

  // 获取小组详情
  getGroup: async (id: string): Promise<Group> => {
    const response = await api.get(`/groups/${id}`)
    return response.data
  },

  // 更新小组
  updateGroup: async (id: string, data: GroupUpdate): Promise<Group> => {
    const response = await api.put(`/groups/${id}`, data)
    return response.data
  },

  // 删除小组
  deleteGroup: async (id: string): Promise<void> => {
    await api.delete(`/groups/${id}`)
  },

  // 获取小组成员
  getGroupMembers: async (id: string): Promise<GroupMember[]> => {
    const response = await api.get(`/groups/${id}/members`)
    return response.data
  },

  // 添加成员
  addGroupMember: async (id: string, data: GroupMemberAdd): Promise<void> => {
    await api.post(`/groups/${id}/members`, data)
  },

  // 移除成员
  removeGroupMember: async (id: string, personId: string): Promise<void> => {
    await api.delete(`/groups/${id}/members/${personId}`)
  },

  // 更新小组积分
  updateGroupScore: async (id: string, data: GroupScoreChange): Promise<GroupScoreRecord> => {
    const response = await api.post(`/groups/${id}/score`, data)
    return response.data
  },

  // 获取小组积分记录
  getGroupScoreRecords: async (id: string): Promise<GroupScoreRecord[]> => {
    const response = await api.get(`/groups/${id}/score-records`)
    return response.data
  },
}
