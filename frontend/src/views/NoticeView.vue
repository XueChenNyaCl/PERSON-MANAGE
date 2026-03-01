<template>
  <div class="notice-container">
    <el-card>
      <template #header>
        <div class="card-header">
          <span>通知管理</span>
          <el-button v-if="canManage" type="primary" @click="handleAdd">发布通知</el-button>
        </div>
      </template>
      <el-form :inline="true" :model="searchForm" class="search-form">
        <el-form-item label="标题">
          <el-input v-model="searchForm.search" placeholder="请输入标题" clearable></el-input>
        </el-form-item>
        <el-form-item label="类型">
          <el-select v-model="searchForm.target_type" placeholder="请选择类型" clearable style="width: 120px;">
            <el-option label="学校公告" value="school"></el-option>
            <el-option label="班级通知" value="class"></el-option>
            <el-option label="部门通知" value="department"></el-option>
          </el-select>
        </el-form-item>
        <el-form-item>
          <el-button type="primary" @click="handleSearch">搜索</el-button>
          <el-button @click="resetSearch">重置</el-button>
        </el-form-item>
      </el-form>
      <el-table :data="noticeList" style="width: 100%" v-loading="loading">
        <el-table-column prop="title" label="标题">
          <template #default="scope">
            <el-tag v-if="scope.row.is_important" type="danger" size="small" style="margin-right: 5px;">重要</el-tag>
            {{ scope.row.title }}
          </template>
        </el-table-column>
        <el-table-column prop="target_type" label="类型" width="100">
          <template #default="scope">
            {{ getTypeText(scope.row.target_type) }}
          </template>
        </el-table-column>
        <el-table-column prop="author_name" label="发布人" width="120"></el-table-column>
        <el-table-column prop="created_at" label="发布时间" width="180">
          <template #default="scope">
            {{ formatDate(scope.row.created_at) }}
          </template>
        </el-table-column>
        <el-table-column label="操作" width="200">
          <template #default="scope">
            <el-button type="primary" size="small" @click="handleView(scope.row)">查看</el-button>
            <el-button v-if="canManage" type="primary" size="small" @click="handleEdit(scope.row)">编辑</el-button>
            <el-button v-if="canManage" type="danger" size="small" @click="handleDelete(scope.row.id)">删除</el-button>
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
    <el-dialog v-model="dialogVisible" :title="isEdit ? '编辑通知' : '发布通知'" width="600px">
      <el-form :model="form" label-width="80px" :rules="rules" ref="formRef">
        <el-form-item label="标题" prop="title">
          <el-input v-model="form.title" placeholder="请输入标题"></el-input>
        </el-form-item>
        <el-form-item label="类型" prop="target_type">
          <el-select v-model="form.target_type" placeholder="选择类型" style="width: 100%;">
            <el-option label="学校公告" value="school"></el-option>
            <el-option label="班级通知" value="class"></el-option>
            <el-option label="部门通知" value="department"></el-option>
          </el-select>
        </el-form-item>
        <el-form-item label="重要通知">
          <el-switch v-model="form.is_important" active-text="是" inactive-text="否"></el-switch>
        </el-form-item>
        <el-form-item label="内容" prop="content">
          <el-input v-model="form.content" type="textarea" rows="6" placeholder="请输入通知内容"></el-input>
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="dialogVisible = false">取消</el-button>
        <el-button type="primary" @click="submitForm">确定</el-button>
      </template>
    </el-dialog>

    <!-- 查看对话框 -->
    <el-dialog v-model="viewDialogVisible" title="通知详情" width="600px">
      <div v-if="currentNotice" class="notice-detail">
        <h3>
          <el-tag v-if="currentNotice.is_important" type="danger" size="small">重要</el-tag>
          {{ currentNotice.title }}
        </h3>
        <div class="notice-meta">
          <span>类型: {{ getTypeText(currentNotice.target_type) }}</span>
          <span>发布人: {{ currentNotice.author_name }}</span>
          <span>发布时间: {{ formatDate(currentNotice.created_at) }}</span>
        </div>
        <div class="notice-content">{{ currentNotice.content }}</div>
      </div>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { noticeApi, NoticeResponse, NoticeCreate, NoticeUpdate } from '../api/notice'
import { useAuthStore } from '../store/auth'

const authStore = useAuthStore()

// 权限检查
const canManage = computed(() => {
  return authStore.hasPermission('notice.manage') || authStore.hasPermission('notice.create')
})

const loading = ref(false)
const noticeList = ref<NoticeResponse[]>([])

const searchForm = ref({
  search: '',
  target_type: ''
})

const currentPage = ref(1)
const pageSize = ref(10)
const total = ref(0)

const dialogVisible = ref(false)
const viewDialogVisible = ref(false)
const isEdit = ref(false)
const currentId = ref('')
const currentNotice = ref<NoticeResponse | null>(null)
const formRef = ref()

const form = ref({
  title: '',
  content: '',
  target_type: 'school',
  is_important: false
})

const rules = {
  title: [{ required: true, message: '请输入标题', trigger: 'blur' }],
  target_type: [{ required: true, message: '请选择类型', trigger: 'change' }],
  content: [{ required: true, message: '请输入内容', trigger: 'blur' }]
}

const getTypeText = (type: string) => {
  const map: Record<string, string> = {
    'school': '学校公告',
    'class': '班级通知',
    'department': '部门通知'
  }
  return map[type] || type
}

const formatDate = (dateStr: string) => {
  if (!dateStr) return '-'
  const date = new Date(dateStr)
  return date.toLocaleString('zh-CN')
}

const loadNoticeList = async () => {
  loading.value = true
  try {
    const response = await noticeApi.list({
      page: currentPage.value,
      limit: pageSize.value,
      search: searchForm.value.search || undefined,
      target_type: searchForm.value.target_type || undefined
    })
    noticeList.value = response.data.items
    total.value = response.data.total
  } catch (error) {
    console.error('加载通知数据失败:', error)
    ElMessage.error('加载通知数据失败')
  } finally {
    loading.value = false
  }
}

const handleSearch = () => {
  currentPage.value = 1
  loadNoticeList()
}

const resetSearch = () => {
  searchForm.value = { search: '', target_type: '' }
  handleSearch()
}

const handleAdd = () => {
  isEdit.value = false
  currentId.value = ''
  form.value = {
    title: '',
    content: '',
    target_type: 'school',
    is_important: false
  }
  dialogVisible.value = true
}

const handleEdit = (row: NoticeResponse) => {
  isEdit.value = true
  currentId.value = row.id
  form.value = {
    title: row.title,
    content: row.content,
    target_type: row.target_type,
    is_important: row.is_important
  }
  dialogVisible.value = true
}

const handleView = (row: NoticeResponse) => {
  currentNotice.value = row
  viewDialogVisible.value = true
}

const submitForm = async () => {
  if (!formRef.value) return
  
  await formRef.value.validate(async (valid: boolean) => {
    if (valid) {
      try {
        if (isEdit.value) {
          const updateData: NoticeUpdate = {
            title: form.value.title,
            content: form.value.content,
            is_important: form.value.is_important
          }
          await noticeApi.update(currentId.value, updateData)
          ElMessage.success('更新成功')
        } else {
          const createData: NoticeCreate = {
            title: form.value.title,
            content: form.value.content,
            target_type: form.value.target_type,
            is_important: form.value.is_important
          }
          await noticeApi.create(createData)
          ElMessage.success('发布成功')
        }
        dialogVisible.value = false
        loadNoticeList()
      } catch (error: any) {
        console.error('提交失败:', error)
        ElMessage.error(error.response?.data || '操作失败')
      }
    }
  })
}

const handleDelete = async (id: string) => {
  try {
    await ElMessageBox.confirm('确定要删除这条通知吗？', '提示', {
      confirmButtonText: '确定',
      cancelButtonText: '取消',
      type: 'warning'
    })
    await noticeApi.delete(id)
    ElMessage.success('删除成功')
    loadNoticeList()
  } catch (error: any) {
    if (error !== 'cancel') {
      console.error('删除失败:', error)
      ElMessage.error(error.response?.data || '删除失败')
    }
  }
}

const handleSizeChange = (size: number) => {
  pageSize.value = size
  loadNoticeList()
}

const handleCurrentChange = (current: number) => {
  currentPage.value = current
  loadNoticeList()
}

onMounted(() => {
  loadNoticeList()
})
</script>

<style scoped src="@/styles/notice-view.css"></style>
