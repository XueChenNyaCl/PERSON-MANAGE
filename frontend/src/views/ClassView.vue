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
            <el-option label="全部" :value="undefined"></el-option>
            <el-option
              v-for="option in gradeOptions"
              :key="option.value"
              :label="option.label"
              :value="option.value"
            ></el-option>
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
            {{ getGradeLabel(scope.row.grade) }}
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

    <el-dialog v-model="dialogVisible" :title="dialogTitle" width="600px">
      <el-form :model="form" :rules="rules" ref="formRef" label-width="100px">
        <el-form-item label="班级名称" prop="name">
          <el-input v-model="form.name" placeholder="请输入班级名称"></el-input>
        </el-form-item>
        <el-form-item label="年级" prop="grade">
          <el-select v-model="form.grade" placeholder="请选择年级">
            <el-option
              v-for="option in gradeOptions"
              :key="option.value"
              :label="option.label"
              :value="option.value"
            ></el-option>
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
import { classApi, type ClassResponse, type ClassCreate, type ClassQuery } from '../api/class'
import { personApi, type PersonResponse } from '../api/person'
import { useMobile } from '../composables/useMobile'
import { getGradeLabel, gradeOptions } from '../utils/classOptions'

const { isMobile, loadMobileStyle } = useMobile()

onMounted(async () => {
  if (isMobile.value) {
    await loadMobileStyle('view-common-mobile')
  }
})

const loading = ref(false)
const submitting = ref(false)
const teachersLoading = ref(false)
const classList = ref<ClassResponse[]>([])
const total = ref(0)
const currentPage = ref(1)
const pageSize = ref(20)
const searchForm = ref<{ search: string; grade?: number }>({
  search: '',
  grade: undefined
})
const dialogVisible = ref(false)
const dialogTitle = ref('新增班级')
const formRef = ref<FormInstance>()
const editingId = ref<string>('')
const teachers = ref<PersonResponse[]>([])

const form = reactive<ClassCreate>({
  name: '',
  grade: 1,
  teacher_id: '',
  academic_year: ''
})

const rules = reactive({
  name: [{ required: true, message: '请输入班级名称', trigger: 'blur' }],
  grade: [{ required: true, message: '请选择年级', trigger: 'change' }],
  academic_year: [{ required: true, message: '请输入学年', trigger: 'blur' }]
})

const loadClasses = async () => {
  loading.value = true
  try {
    console.log('Loading classes...')
    const query: ClassQuery = {
      page: currentPage.value,
      limit: pageSize.value,
      search: searchForm.value.search,
      grade: searchForm.value.grade
    }
    const response = await classApi.list(query)
    console.log('Class API response:', response)
    classList.value = response.data.items
    total.value = response.data.total
    console.log('Loaded classes:', classList.value)
  } catch (error) {
    ElMessage.error('加载班级列表失败')
    console.error('Error loading classes:', error)
    classList.value = []
    total.value = 0
  } finally {
    loading.value = false
  }
}

const loadTeachers = async () => {
  teachersLoading.value = true
  try {
    console.log('Loading teachers...')
    const response = await personApi.list({
      page: 1,
      limit: 100,
      type: 'teacher'
    })
    console.log('Teachers API response:', response)
    teachers.value = response.data.items
    console.log('Loaded teachers:', teachers.value)
  } catch (error) {
    console.error('Error loading teachers:', error)
    teachers.value = []
  } finally {
    teachersLoading.value = false
  }
}

const handleSearch = () => {
  currentPage.value = 1
  loadClasses()
}

const handleSizeChange = (size: number) => {
  pageSize.value = size
  loadClasses()
}

const handleCurrentChange = (current: number) => {
  currentPage.value = current
  loadClasses()
}

const handleAdd = () => {
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

const handleEdit = async (row: ClassResponse) => {
  editingId.value = row.id
  dialogTitle.value = '编辑班级'

  Object.assign(form, {
    name: row.name,
    grade: row.grade,
    teacher_id: row.teacher_id || '',
    academic_year: row.academic_year
  })

  dialogVisible.value = true
}

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

const handleSubmit = async () => {
  if (!formRef.value) return

  try {
    await formRef.value.validate()
    submitting.value = true

    if (editingId.value) {
      await classApi.update(editingId.value, form)
      ElMessage.success('更新成功')
    } else {
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

onMounted(() => {
  loadClasses()
  loadTeachers()
})
</script>

<style scoped src="@/styles/class-view.css"></style>