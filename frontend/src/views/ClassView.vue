<template>
  <div class="class-container">
    <el-card>
      <template #header>
        <div class="card-header">
          <span>班级管理</span>
          <el-button type="primary" @click="handleAdd">新增班级</el-button>
        </div>
      </template>
      <el-form :inline="true" :model="searchForm" class="search-form">
        <el-form-item label="班级名称">
          <el-input v-model="searchForm.search" placeholder="请输入班级名称"></el-input>
        </el-form-item>
        <el-form-item label="年级">
          <el-select v-model="searchForm.grade" placeholder="请选择年级">
            <el-option label="全部" value=""></el-option>
            <el-option label="一年级" value="1"></el-option>
            <el-option label="二年级" value="2"></el-option>
            <el-option label="三年级" value="3"></el-option>
            <el-option label="四年级" value="4"></el-option>
            <el-option label="五年级" value="5"></el-option>
            <el-option label="六年级" value="6"></el-option>
            <el-option label="七年级" value="7"></el-option>
            <el-option label="八年级" value="8"></el-option>
            <el-option label="九年级" value="9"></el-option>
          </el-select>
        </el-form-item>
        <el-form-item>
          <el-button type="primary" @click="handleSearch">搜索</el-button>
        </el-form-item>
      </el-form>
      <el-table :data="classList" style="width: 100%" v-loading="loading">
        <el-table-column label="ID" width="180">
          <template #default="scope">
            <div class="id-cell">{{ scope.row.id }}</div>
          </template>
        </el-table-column>
        <el-table-column prop="name" label="班级名称"></el-table-column>
        <el-table-column prop="grade" label="年级" width="100">
          <template #default="scope">
            {{ scope.row.grade }}年级
          </template>
        </el-table-column>
        <el-table-column prop="teacher_name" label="班主任"></el-table-column>
        <el-table-column prop="academic_year" label="学年" width="120"></el-table-column>
        <el-table-column prop="created_at" label="创建时间" width="180"></el-table-column>
        <el-table-column label="操作" width="150">
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

    <!-- 新增/编辑弹窗 -->
    <el-dialog v-model="dialogVisible" :title="dialogTitle" width="600px">
      <el-form :model="form" :rules="rules" ref="formRef" label-width="100px">
        <el-form-item label="班级名称" prop="name">
          <el-input v-model="form.name" placeholder="请输入班级名称"></el-input>
        </el-form-item>
        <el-form-item label="年级" prop="grade">
          <el-select v-model="form.grade" placeholder="请选择年级">
            <el-option label="一年级" value="1"></el-option>
            <el-option label="二年级" value="2"></el-option>
            <el-option label="三年级" value="3"></el-option>
            <el-option label="四年级" value="4"></el-option>
            <el-option label="五年级" value="5"></el-option>
            <el-option label="六年级" value="6"></el-option>
            <el-option label="七年级" value="7"></el-option>
            <el-option label="八年级" value="8"></el-option>
            <el-option label="九年级" value="9"></el-option>
          </el-select>
        </el-form-item>
        <el-form-item label="班主任">
          <el-select v-model="form.teacher_id" placeholder="请选择班主任" :loading="teachersLoading">
            <el-option v-for="teacher in teachers" :key="teacher.id" :label="teacher.name" :value="teacher.id"></el-option>
          </el-select>
        </el-form-item>
        <el-form-item label="学年" prop="academic_year">
          <el-input v-model="form.academic_year" placeholder="例如：2024-2025"></el-input>
        </el-form-item>
      </el-form>
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="dialogVisible = false">取消</el-button>
          <el-button type="primary" @click="handleSubmit" :loading="submitting">确定</el-button>
        </span>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import type { FormInstance } from 'element-plus'
import { classApi, type ClassResponse, type ClassCreate, type ClassUpdate, type ClassQuery } from '../api/class'
import { personApi, type PersonResponse } from '../api/person'

const loading = ref(false)
const submitting = ref(false)
const teachersLoading = ref(false)
const classList = ref<ClassResponse[]>([])
const total = ref(0)
const currentPage = ref(1)
const pageSize = ref(20)
const searchForm = ref({
  search: '',
  grade: ''
})
const dialogVisible = ref(false)
const dialogTitle = ref('新增班级')
const formRef = ref<FormInstance>()
const editingId = ref<string>('')
const teachers = ref<PersonResponse[]>([])

// 表单数据
const form = reactive<ClassCreate>({
  name: '',
  grade: 1,
  teacher_id: '',
  academic_year: ''
})

// 验证规则
const rules = reactive({
  name: [{ required: true, message: '请输入班级名称', trigger: 'blur' }],
  grade: [{ required: true, message: '请选择年级', trigger: 'change' }],
  academic_year: [{ required: true, message: '请输入学年', trigger: 'blur' }]
})

// 加载班级列表
const loadClasses = async () => {
  loading.value = true
  try {
    const query: ClassQuery = {
      page: currentPage.value,
      limit: pageSize.value,
      search: searchForm.value.search,
      grade: searchForm.value.grade ? Number(searchForm.value.grade) : undefined
    }
    const response = await classApi.list(query)
    classList.value = response.items
    total.value = response.total
  } catch (error) {
    ElMessage.error('加载班级列表失败')
    console.error('Error loading classes:', error)
  } finally {
    loading.value = false
  }
}

// 加载教师列表
const loadTeachers = async () => {
  teachersLoading.value = true
  try {
    const response = await personApi.list({ 
      page: 1, 
      limit: 100, 
      type: 'teacher' 
    })
    teachers.value = response.items
  } catch (error) {
    console.error('Error loading teachers:', error)
  } finally {
    teachersLoading.value = false
  }
}

// 搜索
const handleSearch = () => {
  currentPage.value = 1
  loadClasses()
}

// 分页
const handleSizeChange = (size: number) => {
  pageSize.value = size
  loadClasses()
}

const handleCurrentChange = (current: number) => {
  currentPage.value = current
  loadClasses()
}

// 新增
const handleAdd = () => {
  // 重置表单
  Object.assign(form, {
    name: '',
    grade: 1,
    teacher_id: '',
    academic_year: ''
  })
  editingId.value = ''
  dialogTitle.value = '新增班级'
  dialogVisible.value = true
}

// 编辑
const handleEdit = async (row: ClassResponse) => {
  editingId.value = row.id
  dialogTitle.value = '编辑班级'
  
  // 填充表单数据
  Object.assign(form, {
    name: row.name,
    grade: row.grade,
    teacher_id: row.teacher_id || '',
    academic_year: row.academic_year
  })
  
  dialogVisible.value = true
}

// 删除
const handleDelete = async (id: string) => {
  try {
    await ElMessageBox.confirm('确定要删除该班级吗？', '警告', {
      confirmButtonText: '确定',
      cancelButtonText: '取消',
      type: 'warning'
    })
    
    await classApi.delete(id)
    ElMessage.success('删除成功')
    loadClasses()
  } catch (error) {
    if (error !== 'cancel') {
      ElMessage.error('删除失败')
      console.error('Error deleting class:', error)
    }
  }
}

// 提交
const handleSubmit = async () => {
  if (!formRef.value) return
  
  try {
    await formRef.value.validate()
    submitting.value = true
    
    if (editingId.value) {
      // 更新
      await classApi.update(editingId.value, form)
      ElMessage.success('更新成功')
    } else {
      // 创建
      await classApi.create(form)
      ElMessage.success('创建成功')
    }
    
    dialogVisible.value = false
    loadClasses()
  } catch (error) {
    if (error !== 'cancel') {
      ElMessage.error('操作失败')
      console.error('Error submitting form:', error)
    }
  } finally {
    submitting.value = false
  }
}

// 初始化
onMounted(() => {
  loadClasses()
  loadTeachers()
})
</script>

<style scoped>
.class-container {
  padding: 20px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.search-form {
  margin-bottom: 20px;
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
}

/* ID列滚动样式 */
.id-cell {
  max-width: 100%;
  overflow-x: auto;
  white-space: nowrap;
  text-overflow: ellipsis;
  display: block;
  /* 隐藏滚动条但保留功能 */
  scrollbar-width: thin;
  scrollbar-color: #c1c1c1 #f1f1f1;
}

/* 自定义滚动条样式 */
.id-cell::-webkit-scrollbar {
  height: 4px;
}

.id-cell::-webkit-scrollbar-track {
  background: #f1f1f1;
  border-radius: 4px;
}

.id-cell::-webkit-scrollbar-thumb {
  background: #c1c1c1;
  border-radius: 4px;
}

.id-cell::-webkit-scrollbar-thumb:hover {
  background: #a8a8a8;
}
</style>