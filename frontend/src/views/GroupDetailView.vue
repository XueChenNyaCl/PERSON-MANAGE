<template>
  <div class="group-detail-container">
    <el-card v-loading="loading">
      <template #header>
        <div class="card-header">
          <div class="header-left">
            <el-button @click="goBack" :icon="ArrowLeft">返回</el-button>
            <span class="title">{{ group.name }}</span>
          </div>
          <div class="header-right">
            <el-tag :type="group.score >= 0 ? 'success' : 'danger'" size="large" class="score-tag">
              积分: {{ group.score }}
            </el-tag>
          </div>
        </div>
      </template>
      
      <!-- 小组信息 -->
      <div class="group-info-section">
        <el-descriptions :column="2" border>
          <el-descriptions-item label="所属班级">{{ group.class_name || '-' }}</el-descriptions-item>
          <el-descriptions-item label="成员数量">{{ group.member_count }}人</el-descriptions-item>
          <el-descriptions-item label="创建时间">{{ formatDate(group.created_at) }}</el-descriptions-item>
          <el-descriptions-item label="更新时间">{{ formatDate(group.updated_at) }}</el-descriptions-item>
          <el-descriptions-item label="描述" :span="2">{{ group.description || '暂无描述' }}</el-descriptions-item>
        </el-descriptions>
      </div>
      
      <!-- 成员管理 -->
      <div class="section">
        <div class="section-header">
          <h3>小组成员</h3>
          <el-button v-if="hasClassPermission('group.update.member', group.class_id)" type="primary" @click="openAddMemberDialog">
            添加成员
          </el-button>
        </div>
        
        <el-table :data="members" style="width: 100%" v-loading="membersLoading">
          <el-table-column prop="name" label="姓名" width="120"></el-table-column>
          <el-table-column prop="student_no" label="学号" width="120"></el-table-column>
          <el-table-column label="性别" width="80">
            <template #default="scope">
              {{ scope.row.gender === 1 ? '男' : scope.row.gender === 2 ? '女' : '未知' }}
            </template>
          </el-table-column>
          <el-table-column prop="phone" label="电话" width="120"></el-table-column>
          <el-table-column prop="status" label="状态"></el-table-column>
          <el-table-column label="操作" width="100" v-if="hasClassPermission('group.update.member', group.class_id)">
            <template #default="scope">
              <el-button type="danger" size="small" @click="removeMember(scope.row)">移除</el-button>
            </template>
          </el-table-column>
        </el-table>
        
        <el-empty v-if="members.length === 0 && !membersLoading" description="暂无成员"></el-empty>
      </div>
      
      <!-- 积分管理 -->
      <div class="section">
        <div class="section-header">
          <h3>积分记录</h3>
          <el-button v-if="hasClassPermission('group.update.score', group.class_id)" type="primary" @click="showScoreDialog = true">
            调整积分
          </el-button>
        </div>
        
        <el-table :data="scoreRecords" style="width: 100%" v-loading="recordsLoading">
          <el-table-column label="积分变化" width="120">
            <template #default="scope">
              <span :class="scope.row.score_change >= 0 ? 'score-plus' : 'score-minus'">
                {{ scope.row.score_change >= 0 ? '+' : '' }}{{ scope.row.score_change }}
              </span>
            </template>
          </el-table-column>
          <el-table-column prop="reason" label="原因"></el-table-column>
          <el-table-column prop="created_at" label="时间" width="180">
            <template #default="scope">
              {{ formatDate(scope.row.created_at) }}
            </template>
          </el-table-column>
        </el-table>
        
        <el-empty v-if="scoreRecords.length === 0 && !recordsLoading" description="暂无积分记录"></el-empty>
      </div>
    </el-card>
    
    <!-- 添加成员对话框 -->
    <el-dialog v-model="showAddMemberDialog" title="添加成员" width="700px">
      <div class="add-member-container">
        <!-- 搜索栏 -->
        <div class="search-bar">
          <el-input
            v-model="studentSearchQuery"
            placeholder="搜索学生姓名或学号"
            clearable
            @input="filterStudents"
          >
            <template #prefix>
              <el-icon><Search /></el-icon>
            </template>
          </el-input>
        </div>
        
        <!-- 学生列表 -->
        <div class="student-list-container">
          <el-table
            :data="filteredStudents"
            style="width: 100%"
            height="400"
            v-loading="studentsLoading"
            @selection-change="handleSelectionChange"
            ref="studentTableRef"
          >
            <el-table-column type="selection" width="55"></el-table-column>
            <el-table-column prop="name" label="姓名" width="100"></el-table-column>
            <el-table-column prop="student_no" label="学号" width="120"></el-table-column>
            <el-table-column label="性别" width="80">
              <template #default="scope">
                {{ scope.row.gender === 1 ? '男' : scope.row.gender === 2 ? '女' : '未知' }}
              </template>
            </el-table-column>
            <el-table-column prop="status" label="状态"></el-table-column>
          </el-table>
        </div>
        
        <!-- 已选择提示 -->
        <div class="selected-info" v-if="selectedStudents.length > 0">
          已选择 {{ selectedStudents.length }} 名学生
        </div>
      </div>
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="showAddMemberDialog = false">取消</el-button>
          <el-button type="primary" @click="submitAddMembers" :loading="submitLoading" :disabled="selectedStudents.length === 0">
            确定添加
          </el-button>
        </span>
      </template>
    </el-dialog>
    
    <!-- 调整积分对话框 -->
    <el-dialog v-model="showScoreDialog" title="调整积分" width="500px">
      <el-form :model="scoreForm" :rules="scoreRules" ref="scoreFormRef" label-width="80px">
        <el-form-item label="积分变化" prop="score_change">
          <el-input-number v-model="scoreForm.score_change" :min="-100" :max="100" style="width: 100%"></el-input-number>
        </el-form-item>
        <el-form-item label="原因" prop="reason">
          <el-input v-model="scoreForm.reason" type="textarea" rows="3" placeholder="请输入积分变化原因"></el-input>
        </el-form-item>
      </el-form>
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="showScoreDialog = false">取消</el-button>
          <el-button type="primary" @click="submitScoreChange" :loading="submitLoading">确定</el-button>
        </span>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { ElMessage, ElMessageBox } from 'element-plus'
import { ArrowLeft, Search } from '@element-plus/icons-vue'
import { groupApi, type Group, type GroupMember, type GroupScoreRecord, type GroupScoreChange } from '../api/group'
import { personApi, type PersonResponse } from '../api/person'
import { useAuthStore } from '../store/auth'

const route = useRoute()
const router = useRouter()
const authStore = useAuthStore()

// 权限检查 - 通用权限


// 权限检查 - 班级特定权限
const hasClassPermission = (permission: string, classId: string): boolean => {
  return authStore.hasClassPermission(permission, classId)
}

// 小组ID
const groupId = route.params.id as string

// 加载状态
const loading = ref(false)
const membersLoading = ref(false)
const recordsLoading = ref(false)
const submitLoading = ref(false)
const studentsLoading = ref(false)

// 小组信息
const group = ref<Group>({
  id: '',
  class_id: '',
  name: '',
  description: '',
  score: 0,
  member_count: 0,
  created_at: '',
  updated_at: ''
})

// 成员列表
const members = ref<GroupMember[]>([])

// 积分记录
const scoreRecords = ref<GroupScoreRecord[]>([])

// 可用学生列表
const availableStudents = ref<PersonResponse[]>([])
const filteredStudents = ref<PersonResponse[]>([])
const selectedStudents = ref<PersonResponse[]>([])
const studentSearchQuery = ref('')


// 对话框相关
const showAddMemberDialog = ref(false)
const showScoreDialog = ref(false)
const scoreFormRef = ref()

const scoreForm = reactive<GroupScoreChange>({
  score_change: 0,
  reason: ''
})

const scoreRules = {
  score_change: [
    { required: true, message: '请输入积分变化', trigger: 'blur' }
  ],
  reason: [
    { required: true, message: '请输入原因', trigger: 'blur' },
    { min: 2, max: 200, message: '长度在 2 到 200 个字符', trigger: 'blur' }
  ]
}

// 加载小组详情
const loadGroupDetail = async () => {
  loading.value = true
  try {
    const data = await groupApi.getGroup(groupId)
    group.value = data
  } catch (error) {
    ElMessage.error('加载小组详情失败')
    console.error('Error loading group detail:', error)
  } finally {
    loading.value = false
  }
}

// 加载成员列表
const loadMembers = async () => {
  membersLoading.value = true
  try {
    const data = await groupApi.getGroupMembers(groupId)
    members.value = data
  } catch (error) {
    ElMessage.error('加载成员列表失败')
    console.error('Error loading members:', error)
  } finally {
    membersLoading.value = false
  }
}

// 加载积分记录
const loadScoreRecords = async () => {
  recordsLoading.value = true
  try {
    const data = await groupApi.getGroupScoreRecords(groupId)
    scoreRecords.value = data
  } catch (error) {
    ElMessage.error('加载积分记录失败')
    console.error('Error loading score records:', error)
  } finally {
    recordsLoading.value = false
  }
}

// 加载可用学生
const loadAvailableStudents = async () => {
  if (!group.value.class_id) return
  
  studentsLoading.value = true
  try {
    const response = await personApi.list({
      page: 1,
      limit: 1000,
      type: 'student',
      class_id: group.value.class_id
    })
    
    // 过滤掉已在小组中的学生
    const memberIds = members.value.map(m => m.id)
    availableStudents.value = response.data.items.filter((s: PersonResponse) => !memberIds.includes(s.id))
    filteredStudents.value = [...availableStudents.value]
  } catch (error) {
    console.error('Error loading available students:', error)
    ElMessage.error('加载学生列表失败')
  } finally {
    studentsLoading.value = false
  }
}

// 搜索过滤学生
const filterStudents = () => {
  const query = studentSearchQuery.value.toLowerCase().trim()
  if (!query) {
    filteredStudents.value = [...availableStudents.value]
  } else {
    filteredStudents.value = availableStudents.value.filter((student: PersonResponse) => {
      const nameMatch = student.name.toLowerCase().includes(query)
      const studentNoMatch = 'student_no' in student && (student as any).student_no?.toLowerCase().includes(query)
      return nameMatch || studentNoMatch
    })
  }
}

// 处理选择变化
const handleSelectionChange = (selection: PersonResponse[]) => {
  selectedStudents.value = selection
}

// 打开添加成员对话框
const openAddMemberDialog = () => {
  selectedStudents.value = []
  studentSearchQuery.value = ''
  filterStudents()
  showAddMemberDialog.value = true
}

// 批量添加成员
const submitAddMembers = async () => {
  if (selectedStudents.value.length === 0) {
    ElMessage.warning('请至少选择一名学生')
    return
  }
  
  submitLoading.value = true
  let successCount = 0
  let failCount = 0
  
  for (const student of selectedStudents.value) {
    try {
      await groupApi.addGroupMember(groupId, { person_id: student.id })
      successCount++
    } catch (error: any) {
      failCount++
      console.error(`添加成员 ${student.name} 失败:`, error)
    }
  }
  
  submitLoading.value = false
  
  if (successCount > 0) {
    ElMessage.success(`成功添加 ${successCount} 名成员`)
    showAddMemberDialog.value = false
    selectedStudents.value = []
    studentSearchQuery.value = ''
    await loadMembers()
    await loadGroupDetail()
    await loadAvailableStudents()
  }
  
  if (failCount > 0) {
    ElMessage.error(`${failCount} 名成员添加失败`)
  }
}

// 移除成员
const removeMember = async (member: GroupMember) => {
  try {
    await ElMessageBox.confirm(
      `确定要移除成员 "${member.name}" 吗？`,
      '确认移除',
      {
        confirmButtonText: '确定',
        cancelButtonText: '取消',
        type: 'warning'
      }
    )
    
    await groupApi.removeGroupMember(groupId, member.id)
    ElMessage.success('移除成员成功')
    await loadMembers()
    await loadGroupDetail()
  } catch (error: any) {
    if (error !== 'cancel') {
      ElMessage.error(error.response?.data?.message || '移除成员失败')
    }
  }
}

// 调整积分
const submitScoreChange = async () => {
  if (!scoreFormRef.value) return
  
  await scoreFormRef.value.validate(async (valid: boolean) => {
    if (!valid) return
    
    submitLoading.value = true
    try {
      await groupApi.updateGroupScore(groupId, scoreForm)
      ElMessage.success('调整积分成功')
      showScoreDialog.value = false
      scoreForm.score_change = 0
      scoreForm.reason = ''
      await loadScoreRecords()
      await loadGroupDetail()
    } catch (error: any) {
      ElMessage.error(error.response?.data?.message || '调整积分失败')
    } finally {
      submitLoading.value = false
    }
  })
}

// 返回
const goBack = () => {
  router.back()
}

// 格式化日期
const formatDate = (dateStr: string) => {
  if (!dateStr) return '-'
  const date = new Date(dateStr)
  return date.toLocaleString('zh-CN')
}

// 初始化
onMounted(async () => {
  await loadGroupDetail()
  await Promise.all([
    loadMembers(),
    loadScoreRecords()
  ])
  await loadAvailableStudents()
})
</script>

<style scoped>
.group-detail-container {
  padding: 20px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 15px;
}

.title {
  font-size: 18px;
  font-weight: bold;
  color: #303133;
}

.score-tag {
  font-size: 16px;
  font-weight: bold;
}

.group-info-section {
  margin-bottom: 30px;
}

.section {
  margin-top: 30px;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 15px;
}

.section-header h3 {
  margin: 0;
  font-size: 16px;
  color: #303133;
}

.score-plus {
  color: #67c23a;
  font-weight: bold;
}

.score-minus {
  color: #f56c6c;
  font-weight: bold;
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
}

.add-member-container {
  display: flex;
  flex-direction: column;
  gap: 15px;
}

.search-bar {
  margin-bottom: 10px;
}

.student-list-container {
  border: 1px solid #ebeef5;
  border-radius: 4px;
}

.selected-info {
  padding: 10px;
  background-color: #f5f7fa;
  border-radius: 4px;
  color: #409eff;
  font-weight: bold;
  text-align: center;
}
</style>
