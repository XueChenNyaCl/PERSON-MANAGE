<template>
  <div class="department-container">
    <el-card>
      <template #header>
        <div class="card-header">
          <span>部门管理</span>
          <el-button type="primary" @click="handleAdd">新增部门</el-button>
        </div>
      </template>
      <el-form :inline="true" :model="searchForm" class="search-form">
        <el-form-item label="部门名称">
          <el-input v-model="searchForm.search" placeholder="请输入部门名称"></el-input>
        </el-form-item>
        <el-form-item>
          <el-button type="primary" @click="handleSearch">搜索</el-button>
        </el-form-item>
      </el-form>
      <el-table :data="departmentList" style="width: 100%" v-loading="loading">
        <el-table-column label="ID" width="180">
          <template #default="scope">
            <div class="id-cell">{{ scope.row.id }}</div>
          </template>
        </el-table-column>
        <el-table-column prop="name" label="部门名称"></el-table-column>
        <el-table-column prop="parent_name" label="上级部门"></el-table-column>
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
        <el-form-item label="部门名称" prop="name">
          <el-input v-model="form.name" placeholder="请输入部门名称"></el-input>
        </el-form-item>
        <el-form-item label="上级部门">
          <el-select v-model="form.parent_id" placeholder="请选择上级部门">
            <el-option label="无（顶级部门）" value=""></el-option>
            <el-option v-for="dept in departmentList" :key="dept.id" :label="dept.name" :value="dept.id" :disabled="isParentDisabled(dept.id)"></el-option>
          </el-select>
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
import { departmentApi, type DepartmentResponse, type DepartmentCreate, type DepartmentQuery } from '../api/department'

const loading = ref(false)
const submitting = ref(false)
const departmentList = ref<DepartmentResponse[]>([])
const total = ref(0)
const currentPage = ref(1)
const pageSize = ref(20)
const searchForm = ref({
  search: ''
})
const dialogVisible = ref(false)
const dialogTitle = ref('新增部门')
const formRef = ref<FormInstance>()
const editingId = ref<string>('')

// 表单数据
const form = reactive<DepartmentCreate>({
  name: '',
  parent_id: ''
})

// 验证规则
const rules = reactive({
  name: [{ required: true, message: '请输入部门名称', trigger: 'blur' }]
})

// 加载部门列表
const loadDepartments = async () => {
  loading.value = true
  try {
    console.log('Loading departments...')
    const query: DepartmentQuery = {
      page: currentPage.value,
      limit: pageSize.value,
      search: searchForm.value.search
    }
    const response = await departmentApi.list(query)
    console.log('Department API response:', response)
    departmentList.value = response.data.items
    total.value = response.data.total
    console.log('Loaded departments:', departmentList.value)
  } catch (error) {
    ElMessage.error('加载部门列表失败')
    console.error('Error loading departments:', error)
    departmentList.value = []
    total.value = 0
  } finally {
    loading.value = false
  }
}

// 搜索
const handleSearch = () => {
  currentPage.value = 1
  loadDepartments()
}

// 分页
const handleSizeChange = (size: number) => {
  pageSize.value = size
  loadDepartments()
}

const handleCurrentChange = (current: number) => {
  currentPage.value = current
  loadDepartments()
}

// 新增
const handleAdd = () => {
  // 重置表单
  Object.assign(form, {
    name: '',
    parent_id: ''
  })
  editingId.value = ''
  dialogTitle.value = '新增部门'
  dialogVisible.value = true
}

// 编辑
const handleEdit = async (row: DepartmentResponse) => {
  editingId.value = row.id
  dialogTitle.value = '编辑部门'
  
  // 填充表单数据
  Object.assign(form, {
    name: row.name,
    parent_id: row.parent_id || ''
  })
  
  dialogVisible.value = true
}

// 删除
const handleDelete = async (id: string) => {
  try {
    await ElMessageBox.confirm('确定要删除该部门吗？', '警告', {
      confirmButtonText: '确定',
      cancelButtonText: '取消',
      type: 'warning'
    })
    
    await departmentApi.delete(id)
    ElMessage.success('删除成功')
    loadDepartments()
  } catch (error) {
    if (error !== 'cancel') {
      ElMessage.error('删除失败')
      console.error('Error deleting department:', error)
    }
  }
}

// 检查上级部门是否禁用（不能选择自己或自己的子部门）
const isParentDisabled = (deptId: string) => {
  // 不能选择自己作为上级
  if (editingId.value && deptId === editingId.value) {
    return true
  }
  
  // 不能选择自己的子部门作为上级（避免循环依赖）
  if (editingId.value) {
    return hasChildDepartment(editingId.value, deptId)
  }
  
  return false
}

// 检查部门A是否有子部门B（递归检查）
const hasChildDepartment = (parentId: string, childId: string): boolean => {
  // 找到当前部门
  const currentDept = departmentList.value.find(dept => dept.id === parentId)
  if (!currentDept) return false
  
  // 直接检查子部门
  for (const dept of departmentList.value) {
    if (dept.parent_id === parentId) {
      if (dept.id === childId) {
        return true
      }
      // 递归检查下级部门
      if (hasChildDepartment(dept.id, childId)) {
        return true
      }
    }
  }
  
  return false
}

// 提交
const handleSubmit = async () => {
  if (!formRef.value) return
  
  try {
    await formRef.value.validate()
    submitting.value = true
    
    if (editingId.value) {
      // 更新
      await departmentApi.update(editingId.value, form)
      ElMessage.success('更新成功')
    } else {
      // 创建
      await departmentApi.create(form)
      ElMessage.success('创建成功')
    }
    
    dialogVisible.value = false
    loadDepartments()
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
  loadDepartments()
})
</script>

<style scoped>
.department-container {
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