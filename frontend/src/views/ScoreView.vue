<template>
  <div class="score-container">
    <el-card>
      <template #header>
        <div class="card-header">
          <span>评分管理</span>
          <el-button v-if="canManage" type="primary" @click="handleAdd">新增评分</el-button>
        </div>
      </template>
      <el-form :inline="true" :model="searchForm" class="search-form">
        <el-form-item label="姓名">
          <el-input v-model="searchForm.name" placeholder="请输入姓名" clearable></el-input>
        </el-form-item>
        <el-form-item label="类型">
          <el-select v-model="searchForm.score_type" placeholder="请选择类型" clearable style="width: 120px;">
            <el-option label="个人分" value="personal"></el-option>
            <el-option label="小组分" value="group"></el-option>
          </el-select>
        </el-form-item>
        <el-form-item>
          <el-button type="primary" @click="handleSearch">搜索</el-button>
          <el-button @click="resetSearch">重置</el-button>
        </el-form-item>
      </el-form>
      <el-table :data="scoreList" style="width: 100%" v-loading="loading">
        <el-table-column prop="person_name" label="姓名" width="100"></el-table-column>
        <el-table-column prop="group_name" label="小组" width="120">
          <template #default="scope">
            {{ scope.row.group_name || '-' }}
          </template>
        </el-table-column>
        <el-table-column prop="score_type" label="类型" width="100">
          <template #default="scope">
            {{ scope.row.score_type === 'personal' ? '个人分' : '小组分' }}
          </template>
        </el-table-column>
        <el-table-column prop="value" label="分数" width="100">
          <template #default="scope">
            <span :class="scope.row.value > 0 ? 'score-positive' : 'score-negative'">
              {{ scope.row.value > 0 ? '+' : '' }}{{ scope.row.value }}
            </span>
          </template>
        </el-table-column>
        <el-table-column prop="reason" label="原因"></el-table-column>
        <el-table-column prop="created_at" label="创建时间" width="180">
          <template #default="scope">
            {{ formatDate(scope.row.created_at) }}
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
    <el-dialog v-model="dialogVisible" :title="isEdit ? '编辑评分' : '新增评分'" width="500px">
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
        <el-form-item label="小组">
          <el-select v-model="form.group_id" clearable placeholder="选择小组（可选）" style="width: 100%;">
            <el-option
              v-for="group in groupList"
              :key="group.id"
              :label="group.name"
              :value="group.id"
            />
          </el-select>
        </el-form-item>
        <el-form-item label="类型" prop="score_type">
          <el-select v-model="form.score_type" placeholder="选择类型" style="width: 100%;">
            <el-option label="个人分" value="personal"></el-option>
            <el-option label="小组分" value="group"></el-option>
          </el-select>
        </el-form-item>
        <el-form-item label="分数" prop="value">
          <el-input-number v-model="form.value" :min="-100" :max="100" style="width: 100%;"></el-input-number>
        </el-form-item>
        <el-form-item label="原因" prop="reason">
          <el-input v-model="form.reason" type="textarea" rows="3" placeholder="请输入评分原因"></el-input>
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
import { scoreApi, ScoreResponse, ScoreCreate, ScoreUpdate } from '../api/score'
import { personApi } from '../api/person'
import { groupApi } from '../api/group'
import { useAuthStore } from '../store/auth'

const authStore = useAuthStore()

// 权限检查
const canManage = computed(() => {
  return authStore.hasPermission('score.manage') || authStore.hasPermission('score.create')
})

const loading = ref(false)
const scoreList = ref<ScoreResponse[]>([])
const personList = ref<{id: string, name: string}[]>([])
const groupList = ref<{id: string, name: string}[]>([])

const searchForm = ref({
  name: '',
  score_type: ''
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
  group_id: undefined as string | undefined,
  score_type: 'personal',
  value: 0,
  reason: ''
})

const rules = {
  person_id: [{ required: true, message: '请选择人员', trigger: 'change' }],
  score_type: [{ required: true, message: '请选择类型', trigger: 'change' }],
  value: [{ required: true, message: '请输入分数', trigger: 'change' }],
  reason: [{ required: true, message: '请输入原因', trigger: 'blur' }]
}

const formatDate = (dateStr: string) => {
  if (!dateStr) return '-'
  const date = new Date(dateStr)
  return date.toLocaleString('zh-CN')
}

const loadScoreList = async () => {
  loading.value = true
  try {
    const response = await scoreApi.list({
      page: currentPage.value,
      limit: pageSize.value,
      score_type: searchForm.value.score_type || undefined
    })
    scoreList.value = response.data.items
    total.value = response.data.total
  } catch (error) {
    console.error('加载评分数据失败:', error)
    ElMessage.error('加载评分数据失败')
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

const loadGroupList = async () => {
  try {
    const groups = await groupApi.list()
    groupList.value = groups.map((g: any) => ({ id: g.id, name: g.name }))
  } catch (error) {
    console.error('加载小组列表失败:', error)
  }
}

const handleSearch = () => {
  currentPage.value = 1
  loadScoreList()
}

const resetSearch = () => {
  searchForm.value = { name: '', score_type: '' }
  handleSearch()
}

const handleAdd = () => {
  isEdit.value = false
  currentId.value = ''
  form.value = {
    person_id: '',
    group_id: undefined,
    score_type: 'personal',
    value: 0,
    reason: ''
  }
  dialogVisible.value = true
}

const handleEdit = (row: ScoreResponse) => {
  isEdit.value = true
  currentId.value = row.id
  form.value = {
    person_id: row.person_id,
    group_id: row.group_id,
    score_type: row.score_type,
    value: row.value,
    reason: row.reason
  }
  dialogVisible.value = true
}

const submitForm = async () => {
  if (!formRef.value) return
  
  await formRef.value.validate(async (valid: boolean) => {
    if (valid) {
      try {
        if (isEdit.value) {
          const updateData: ScoreUpdate = {
            value: form.value.value,
            reason: form.value.reason
          }
          await scoreApi.update(currentId.value, updateData)
          ElMessage.success('更新成功')
        } else {
          const createData: ScoreCreate = {
            person_id: form.value.person_id,
            group_id: form.value.group_id,
            score_type: form.value.score_type,
            value: form.value.value,
            reason: form.value.reason
          }
          await scoreApi.create(createData)
          ElMessage.success('创建成功')
        }
        dialogVisible.value = false
        loadScoreList()
      } catch (error: any) {
        console.error('提交失败:', error)
        ElMessage.error(error.response?.data || '操作失败')
      }
    }
  })
}

const handleDelete = async (id: string) => {
  try {
    await ElMessageBox.confirm('确定要删除这条评分记录吗？', '提示', {
      confirmButtonText: '确定',
      cancelButtonText: '取消',
      type: 'warning'
    })
    await scoreApi.delete(id)
    ElMessage.success('删除成功')
    loadScoreList()
  } catch (error: any) {
    if (error !== 'cancel') {
      console.error('删除失败:', error)
      ElMessage.error(error.response?.data || '删除失败')
    }
  }
}

const handleSizeChange = (size: number) => {
  pageSize.value = size
  loadScoreList()
}

const handleCurrentChange = (current: number) => {
  currentPage.value = current
  loadScoreList()
}

onMounted(() => {
  loadScoreList()
  loadPersonList()
  loadGroupList()
})
</script>

<style scoped src="@/styles/score-view.css"></style>
