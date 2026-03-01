<template>
  <div class="attendance-container">
    <el-card>
      <template #header>
        <div class="card-header">
          <span>考勤管理</span>
          <el-button v-if="canManage" type="primary" @click="handleAdd">新增考勤</el-button>
        </div>
      </template>
      <el-form :inline="true" :model="searchForm" class="search-form">
        <el-form-item label="姓名">
          <el-input v-model="searchForm.name" placeholder="请输入姓名" clearable></el-input>
        </el-form-item>
        <el-form-item label="日期">
          <el-date-picker v-model="searchForm.date" type="date" placeholder="选择日期" value-format="YYYY-MM-DD"></el-date-picker>
        </el-form-item>
        <el-form-item label="状态">
          <el-select v-model="searchForm.status" placeholder="选择状态" clearable style="width: 120px;">
            <el-option label="正常" value="present"></el-option>
            <el-option label="迟到" value="late"></el-option>
            <el-option label="缺勤" value="absent"></el-option>
            <el-option label="早退" value="early_leave"></el-option>
            <el-option label="请假" value="excused"></el-option>
          </el-select>
        </el-form-item>
        <el-form-item>
          <el-button type="primary" @click="handleSearch">搜索</el-button>
          <el-button @click="resetSearch">重置</el-button>
        </el-form-item>
      </el-form>
      <el-table :data="attendanceList" style="width: 100%" v-loading="loading">
        <el-table-column prop="person_name" label="姓名" width="100"></el-table-column>
        <el-table-column prop="date" label="日期" width="120"></el-table-column>
        <el-table-column prop="status" label="状态" width="100">
          <template #default="scope">
            <el-tag :type="getTagType(scope.row.status)">{{ getStatusText(scope.row.status) }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="time" label="时间" width="100">
          <template #default="scope">
            {{ scope.row.time || '-' }}
          </template>
        </el-table-column>
        <el-table-column prop="remark" label="备注">
          <template #default="scope">
            {{ scope.row.remark || '-' }}
          </template>
        </el-table-column>
        <el-table-column label="操作" width="150" v-if="canManage">
          <template #default="scope">
            <el-button type="primary" size="small" @click="handleEdit(scope.row)">编辑</el-button>
            <el-button type="danger" size="small" @click="handleDelete(scope.row.id)">删除</el-button>
          </template>
        </el-table-column>
      </el-table>
      <el-pagination
        v-model:current-page="currentPage"
        v-model:page-size="pageSize"
        :page-sizes="[10, 20, 50, 100]"
        layout="total, sizes, prev, pager, next, jumper"
        :total="total"
        @size-change="handleSizeChange"
        @current-change="handleCurrentChange"
      />
    </el-card>

    <!-- 新增/编辑对话框 -->
    <el-dialog v-model="dialogVisible" :title="isEdit ? '编辑考勤' : '新增考勤'" width="500px">
      <el-form :model="form" label-width="80px" :rules="rules" ref="formRef">
        <el-form-item label="人员" prop="person_id">
          <el-select v-model="form.person_id" filterable placeholder="选择人员" style="width: 100%;">
            <el-option
              v-for="person in personList"
              :key="person.id"
              :label="person.name"
              :value="person.id"
            />
          </el-select>
        </el-form-item>
        <el-form-item label="日期" prop="date">
          <el-date-picker v-model="form.date" type="date" placeholder="选择日期" value-format="YYYY-MM-DD" style="width: 100%;"></el-date-picker>
        </el-form-item>
        <el-form-item label="状态" prop="status">
          <el-select v-model="form.status" placeholder="选择状态" style="width: 100%;">
            <el-option label="正常" value="present"></el-option>
            <el-option label="迟到" value="late"></el-option>
            <el-option label="缺勤" value="absent"></el-option>
            <el-option label="早退" value="early_leave"></el-option>
            <el-option label="请假" value="excused"></el-option>
          </el-select>
        </el-form-item>
        <el-form-item label="时间">
          <el-time-picker v-model="form.time" placeholder="选择时间" value-format="HH:mm:ss" style="width: 100%;"></el-time-picker>
        </el-form-item>
        <el-form-item label="备注">
          <el-input v-model="form.remark" type="textarea" rows="3" placeholder="请输入备注"></el-input>
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="dialogVisible = false">取消</el-button>
        <el-button type="primary" @click="submitForm">确定</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { attendanceApi, AttendanceResponse, AttendanceCreate, AttendanceUpdate } from '../api/attendance'
import { personApi } from '../api/person'
import { useAuthStore } from '../store/auth'

const authStore = useAuthStore()

// 权限检查
const canManage = computed(() => {
  return authStore.hasPermission('attendance.manage') || authStore.hasPermission('attendance.create')
})

const loading = ref(false)
const attendanceList = ref<AttendanceResponse[]>([])
const personList = ref<{id: string, name: string}[]>([])

const searchForm = ref({
  name: '',
  date: '',
  status: ''
})

const currentPage = ref(1)
const pageSize = ref(10)
const total = ref(0)

const dialogVisible = ref(false)
const isEdit = ref(false)
const currentId = ref('')
const formRef = ref()

const form = ref({
  person_id: '',
  date: '',
  status: 'present',
  time: '',
  remark: ''
})

const rules = {
  person_id: [{ required: true, message: '请选择人员', trigger: 'change' }],
  date: [{ required: true, message: '请选择日期', trigger: 'change' }],
  status: [{ required: true, message: '请选择状态', trigger: 'change' }]
}

const getTagType = (status: string) => {
  switch (status) {
    case 'present': return 'success'
    case 'late': return 'warning'
    case 'absent': return 'danger'
    case 'early_leave': return 'info'
    case 'excused': return ''
    default: return ''
  }
}

const getStatusText = (status: string) => {
  const map: Record<string, string> = {
    'present': '正常',
    'late': '迟到',
    'absent': '缺勤',
    'early_leave': '早退',
    'excused': '请假'
  }
  return map[status] || status
}

const loadAttendanceList = async () => {
  loading.value = true
  try {
    const response = await attendanceApi.list({
      page: currentPage.value,
      limit: pageSize.value,
      date: searchForm.value.date || undefined,
      status: searchForm.value.status || undefined
    })
    attendanceList.value = response.data.items
    total.value = response.data.total
  } catch (error) {
    console.error('加载考勤数据失败:', error)
    ElMessage.error('加载考勤数据失败')
  } finally {
    loading.value = false
  }
}

const loadPersonList = async () => {
  try {
    const response = await personApi.list({ page: 1, limit: 1000 })
    personList.value = response.data.items.map((p: any) => ({ id: p.id, name: p.name }))
  } catch (error) {
    console.error('加载人员列表失败:', error)
  }
}

const handleSearch = () => {
  currentPage.value = 1
  loadAttendanceList()
}

const resetSearch = () => {
  searchForm.value = { name: '', date: '', status: '' }
  handleSearch()
}

const handleAdd = () => {
  isEdit.value = false
  currentId.value = ''
  form.value = {
    person_id: '',
    date: '',
    status: 'present',
    time: '',
    remark: ''
  }
  dialogVisible.value = true
}

const handleEdit = (row: AttendanceResponse) => {
  isEdit.value = true
  currentId.value = row.id
  form.value = {
    person_id: row.person_id,
    date: row.date,
    status: row.status,
    time: row.time || '',
    remark: row.remark || ''
  }
  dialogVisible.value = true
}

const submitForm = async () => {
  if (!formRef.value) return
  
  await formRef.value.validate(async (valid: boolean) => {
    if (valid) {
      try {
        if (isEdit.value) {
          const updateData: AttendanceUpdate = {
            status: form.value.status as 'present' | 'late' | 'absent' | 'early_leave' | 'excused',
            time: form.value.time || undefined,
            remark: form.value.remark || undefined
          }
          await attendanceApi.update(currentId.value, updateData)
          ElMessage.success('更新成功')
        } else {
          const createData: AttendanceCreate = {
            person_id: form.value.person_id,
            date: form.value.date,
            status: form.value.status as any,
            time: form.value.time || undefined,
            remark: form.value.remark || undefined
          }
          await attendanceApi.create(createData)
          ElMessage.success('创建成功')
        }
        dialogVisible.value = false
        loadAttendanceList()
      } catch (error: any) {
        console.error('提交失败:', error)
        ElMessage.error(error.response?.data || '操作失败')
      }
    }
  })
}

const handleDelete = async (id: string) => {
  try {
    await ElMessageBox.confirm('确定要删除这条考勤记录吗？', '提示', {
      confirmButtonText: '确定',
      cancelButtonText: '取消',
      type: 'warning'
    })
    await attendanceApi.delete(id)
    ElMessage.success('删除成功')
    loadAttendanceList()
  } catch (error: any) {
    if (error !== 'cancel') {
      console.error('删除失败:', error)
      ElMessage.error(error.response?.data || '删除失败')
    }
  }
}

const handleSizeChange = (size: number) => {
  pageSize.value = size
  loadAttendanceList()
}

const handleCurrentChange = (current: number) => {
  currentPage.value = current
  loadAttendanceList()
}

onMounted(() => {
  loadAttendanceList()
  loadPersonList()
})
</script>

<style scoped src="@/styles/attendance-view.css"></style>
