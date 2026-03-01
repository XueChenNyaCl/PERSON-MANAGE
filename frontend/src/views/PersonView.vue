<template>
  <div class="person-container">
    <el-card>
      <template #header>
        <div class="card-header">
          <span>人员管理</span>
          <el-button type="primary" @click="handleAdd">新增人员</el-button>
        </div>
      </template>
      <el-form :inline="true" :model="searchForm" class="search-form">
        <el-form-item label="姓名">
          <el-input v-model="searchForm.search" placeholder="请输入姓名"></el-input>
        </el-form-item>
        <el-form-item label="类型">
          <el-select v-model="searchForm.type" placeholder="请选择类型">
            <el-option label="全部" value=""></el-option>
            <el-option label="学生" value="student"></el-option>
            <el-option label="教师" value="teacher"></el-option>
            <el-option label="家长" value="parent"></el-option>
          </el-select>
        </el-form-item>
        <el-form-item>
          <el-button type="primary" @click="handleSearch">搜索</el-button>
        </el-form-item>
      </el-form>
      <el-table :data="personList" style="width: 100%" v-loading="loading">
        <el-table-column label="ID" width="180">
          <template #default="scope">
            <div class="id-cell">{{ scope.row.id }}</div>
          </template>
        </el-table-column>
        <el-table-column prop="name" label="姓名"></el-table-column>
        <el-table-column prop="type" label="类型">
          <template #default="scope">
            {{ getTypeName(scope.row) }}
          </template>
        </el-table-column>
        <el-table-column label="编号">
          <template #default="scope">
            {{ getPersonNo(scope.row) }}
          </template>
        </el-table-column>
        <el-table-column label="所属">
          <template #default="scope">
            {{ getPersonBelong(scope.row) }}
          </template>
        </el-table-column>
        <el-table-column label="电话" width="120">
          <template #default="scope">
            <span v-if="canViewSensitiveInfo">{{ scope.row.phone || '未设置' }}</span>
            <span v-else class="sensitive-info-hidden">***</span>
          </template>
        </el-table-column>
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

      <!-- 新增/编辑人员弹窗 -->
      <el-dialog v-model="dialogVisible" :title="dialogTitle" width="700px">
        <el-form :model="form" :rules="rules" ref="formRef" label-width="100px">
          <el-form-item label="姓名" prop="name">
            <el-input v-model="form.name" placeholder="请输入姓名"></el-input>
          </el-form-item>
          <el-form-item label="性别" prop="gender">
            <el-select v-model="form.gender" placeholder="请选择性别">
              <el-option label="男" :value="1"></el-option>
              <el-option label="女" :value="2"></el-option>
              <el-option label="未知" :value="0"></el-option>
            </el-select>
          </el-form-item>
          <el-form-item label="类型" prop="type_">
            <el-select v-model="form.type_" placeholder="请选择类型" @change="handleTypeChange">
              <el-option label="学生" value="student"></el-option>
              <el-option label="教师" value="teacher"></el-option>
              <el-option label="家长" value="parent"></el-option>
            </el-select>
          </el-form-item>
          <el-form-item label="电话">
            <el-input v-model="form.phone" placeholder="请输入电话"></el-input>
          </el-form-item>
          <el-form-item label="邮箱">
            <el-input v-model="form.email" placeholder="请输入邮箱"></el-input>
          </el-form-item>
          <el-form-item label="密码" prop="password">
            <el-input 
              v-model="form.password" 
              type="password" 
              placeholder="请输入密码，留空则使用默认密码123456"
              show-password
              clearable
            ></el-input>
          </el-form-item>
          <el-form-item label="生日">
            <el-date-picker v-model="form.birthday" type="date" placeholder="请选择生日"></el-date-picker>
          </el-form-item>

          <!-- 学生特定字段 -->
          <template v-if="form.type_ === 'student'">
            <el-form-item label="学号" prop="student_no">
              <el-input v-model="form.student_no" placeholder="请输入学号"></el-input>
            </el-form-item>
            <el-form-item label="关联班级">
              <el-select 
                v-model="form.class_id" 
                placeholder="请选择班级" 
                :loading="classesLoading"
                style="width: 200px;"
              >
                <el-option v-for="cls in classes" :key="cls.id" :label="cls.name" :value="cls.id"></el-option>
              </el-select>
            </el-form-item>
            <el-form-item label="入学日期">
              <el-date-picker v-model="form.enrollment_date" type="date" placeholder="请选择入学日期"></el-date-picker>
            </el-form-item>
          </template>

          <!-- 教师特定字段 -->
          <template v-if="form.type_ === 'teacher'">
            <el-form-item label="工号" prop="employee_no">
              <el-input v-model="form.employee_no" placeholder="请输入工号"></el-input>
            </el-form-item>
            <el-form-item label="部门">
              <el-select v-model="form.department_id" placeholder="请选择部门" :loading="departmentsLoading">
                <el-option v-for="dept in departments" :key="dept.id" :label="dept.name" :value="dept.id"></el-option>
              </el-select>
            </el-form-item>
            <el-form-item label="关联班级">
              <div class="classes-container">
                <div v-for="(classItem, index) in form.classes" :key="index" class="class-item">
                  <el-select 
                    v-model="classItem.class_id" 
                    placeholder="请选择班级" 
                    :loading="classesLoading"
                    style="width: 200px; margin-right: 10px;"
                  >
                    <el-option v-for="cls in classes" :key="cls.id" :label="cls.name" :value="cls.id"></el-option>
                  </el-select>
                  <el-checkbox v-model="classItem.is_main_teacher" label="班主任" style="margin-right: 10px;" />
                  <el-button 
                    type="danger" 
                    size="small" 
                    circle 
                    @click="removeClass(index)"
                    :disabled="(form.classes || []).length <= 1"
                  >
                    <el-icon><Delete /></el-icon>
                  </el-button>
                </div>
                <el-button type="primary" size="small" @click="addClass" style="margin-top: 10px;">
                  <el-icon><Plus /></el-icon>
                  添加班级
                </el-button>
              </div>
            </el-form-item>
            <el-form-item label="职称">
              <el-input v-model="form.title" placeholder="请输入职称"></el-input>
            </el-form-item>
            <el-form-item label="入职日期">
              <el-date-picker v-model="form.hire_date" type="date" placeholder="请选择入职日期"></el-date-picker>
            </el-form-item>
          </template>

          <!-- 家长特定字段 -->
          <template v-if="form.type_ === 'parent'">
            <el-form-item label="职业">
              <el-input v-model="form.occupation" placeholder="请输入职业"></el-input>
            </el-form-item>
            <el-form-item label="微信OpenID">
              <el-input v-model="form.wechat_openid" placeholder="请输入微信OpenID"></el-input>
            </el-form-item>
          </template>
        </el-form>
        <template #footer>
          <span class="dialog-footer">
            <el-button @click="dialogVisible = false">取消</el-button>
            <el-button type="primary" @click="handleSubmit" :loading="submitting">确定</el-button>
          </span>
        </template>
      </el-dialog>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import type { FormInstance } from 'element-plus'
import { Delete, Plus } from '@element-plus/icons-vue'
import { personApi, type PersonResponse, type PersonCreate, type PersonQuery, type TeacherClassResponse } from '../api/person'
import { classApi } from '../api/class'
import { departmentApi } from '../api/department'
import { useAuthStore } from '../store/auth'
import '../styles/person-view.css'

// 权限管理
const authStore = useAuthStore()

// 检查是否有查看敏感信息的权限
const canViewSensitiveInfo = computed(() => {
  return authStore.hasPermission('person.sensitive.view')
})

// 人员管理相关
const loading = ref(false)
const submitting = ref(false)
const classesLoading = ref(false)
const departmentsLoading = ref(false)
const personList = ref<PersonResponse[]>([])
const total = ref(0)
const currentPage = ref(1)
const pageSize = ref(20)
const searchForm = ref({
  type: '',
  search: ''
})
const dialogVisible = ref(false)
const dialogTitle = ref('新增人员')
const formRef = ref<FormInstance>()
const editingId = ref<string>('')

// 班级和部门列表（用于人员编辑）
const classes = ref<any[]>([])
const departments = ref<any[]>([])

// 表单数据
const form = reactive<PersonCreate & { password?: string }>({
  name: '',
  gender: 0,
  type_: 'student',
  phone: '',
  email: '',
  password: '',
  birthday: '',
  student_no: '',
  class_id: '',
  enrollment_date: '',
  employee_no: '',
  department_id: '',
  title: '',
  hire_date: '',
  wechat_openid: '',
  occupation: '',
  // 老师关联的多个班级
  classes: []
})

// 验证规则
const rules = reactive({
  name: [{ required: true, message: '请输入姓名', trigger: 'blur' }],
  gender: [{ required: true, message: '请选择性别', trigger: 'change' }],
  type_: [{ required: true, message: '请选择类型', trigger: 'change' }],
  student_no: [
    {
      required: computed(() => form.type_ === 'student'),
      message: '请输入学号',
      trigger: 'blur'
    }
  ],
  employee_no: [
    {
      validator: (_rule: any, value: string, callback: any) => {
        if (form.type_ === 'teacher') {
          if (!value || value.trim() === '') {
            callback(new Error('请输入工号'))
          } else {
            callback()
          }
        } else {
          callback()
        }
      },
      trigger: 'blur'
    }
  ]
})

// 格式化类型名称
const getTypeName = (row: PersonResponse) => {
  switch (row.type) {
    case 'student': return '学生'
    case 'teacher': return '教师'
    case 'parent': return '家长'
    default: return '未知'
  }
}

// 获取人员编号（根据权限决定是否显示敏感信息）
const getPersonNo = (row: PersonResponse) => {
  if (!canViewSensitiveInfo.value) {
    return '***' // 隐藏敏感信息
  }
  
  if (row.type === 'student') {
    return (row as any).student_no || '未设置'
  } else if (row.type === 'teacher') {
    return (row as any).employee_no || '未设置'
  }
  return ''
}

// 获取所属信息
const getPersonBelong = (row: PersonResponse) => {
  if (row.type === 'student') {
    return (row as any).class_name || ''
  } else if (row.type === 'teacher') {
    return (row as any).department_name || ''
  }
  return ''
}

// 加载人员列表
const loadPersons = async () => {
  loading.value = true
  try {
    const query: PersonQuery = {
      page: currentPage.value,
      limit: pageSize.value,
      type: searchForm.value.type,
      search: searchForm.value.search
    }
    console.log('Fetching persons with query:', query)
    const response = await personApi.list(query)
    console.log('Received response:', response)
    console.log('Response data:', response.data)
    console.log('Response items:', response.data.items)
    console.log('Response total:', response.data.total)
    personList.value = response.data.items
    total.value = response.data.total
    console.log('Updated personList:', personList.value)
    console.log('Updated total:', total.value)
  } catch (error) {
    ElMessage.error('加载人员列表失败')
    console.error('Error loading persons:', error)
  } finally {
    loading.value = false
  }
}

// 加载班级列表
const loadClasses = async () => {
  classesLoading.value = true
  try {
    const response = await classApi.list({ page: 1, limit: 100 })
    classes.value = response.data.items
  } catch (error) {
    console.error('Error loading classes:', error)
  } finally {
    classesLoading.value = false
  }
}

// 加载部门列表
const loadDepartments = async () => {
  departmentsLoading.value = true
  try {
    const response = await departmentApi.list({ page: 1, limit: 100 })
    departments.value = response.data.items
  } catch (error) {
    console.error('Error loading departments:', error)
  } finally {
    departmentsLoading.value = false
  }
}

// 添加班级
const addClass = () => {
  form.classes!.push({
    class_id: '',
    is_main_teacher: false
  })
}

// 移除班级
const removeClass = (index: number) => {
  form.classes!.splice(index, 1)
}

// 搜索
const handleSearch = () => {
  currentPage.value = 1
  loadPersons()
}

// 分页
const handleSizeChange = (size: number) => {
  pageSize.value = size
  loadPersons()
}

const handleCurrentChange = (current: number) => {
  currentPage.value = current
  loadPersons()
}

// 新增
const handleAdd = () => {
  // 重置表单
  Object.assign(form, {
    name: '',
    gender: 0,
    type_: 'student',
    phone: '',
    email: '',
    password: '',
    birthday: '',
    student_no: '',
    class_id: '',
    enrollment_date: '',
    employee_no: '',
    department_id: '',
    title: '',
    hire_date: '',
    wechat_openid: '',
    occupation: '',
    // 老师关联的多个班级
    classes: []
  })
  editingId.value = ''
  dialogTitle.value = '新增人员'
  dialogVisible.value = true
}

// 编辑
const handleEdit = async (row: PersonResponse) => {
  editingId.value = row.id
  dialogTitle.value = '编辑人员'
  
  // 加载表单数据
  Object.assign(form, {
    name: row.name,
    gender: row.gender,
    type_: row.type,
    phone: row.phone || '',
    email: row.email || '',
    password: '', // 编辑时密码为空，表示不修改密码
    birthday: row.birthday || '',
    student_no: (row as any).student_no || '',
    class_id: (row as any).class_id || '',
    enrollment_date: (row as any).enrollment_date || '',
    employee_no: (row as any).employee_no || '',
    department_id: (row as any).department_id || '',
    title: (row as any).title || '',
    hire_date: (row as any).hire_date || '',
    wechat_openid: (row as any).wechat_openid || '',
    occupation: (row as any).occupation || '',
    // 老师关联的多个班级
    classes: []
  })
  
  // 如果是教师，加载关联的班级信息
  if (row.type === 'teacher') {
    try {
      const teacherClasses = await personApi.getTeacherClasses(row.id)
      form.classes = teacherClasses.data.map((item: TeacherClassResponse) => ({
        class_id: item.id,
        is_main_teacher: item.is_main_teacher || false
      }))
    } catch (error) {
      console.error('Error loading teacher classes:', error)
      form.classes = []
    }
  }
  
  dialogVisible.value = true
}

// 删除
const handleDelete = async (id: string) => {
  try {
    await ElMessageBox.confirm('确定要删除该人员吗？', '警告', {
      confirmButtonText: '确定',
      cancelButtonText: '取消',
      type: 'warning'
    })
    
    await personApi.delete(id)
    ElMessage.success('删除成功')
    loadPersons()
  } catch (error) {
    if (error !== 'cancel') {
      ElMessage.error('删除失败')
      console.error('Error deleting person:', error)
    }
  }
}

// 类型变更
const handleTypeChange = () => {
  // 清空特定类型的字段
  if (form.type_ !== 'student') {
    form.student_no = ''
    form.class_id = ''
    form.enrollment_date = ''
  }
  if (form.type_ !== 'teacher') {
    form.employee_no = ''
    form.department_id = ''
    form.title = ''
    form.hire_date = ''
    // 清空教师关联的班级
    form.classes = []
  }
  if (form.type_ !== 'parent') {
    form.wechat_openid = ''
    form.occupation = ''
  }
  if (form.type_ === 'teacher') {
    // 如果是教师，确保classes数组至少有一个条目
    if (!form.classes || form.classes.length === 0) {
      form.classes = [{
        class_id: '',
        is_main_teacher: false
      }]
    }
  }
}

// 提交
const handleSubmit = async () => {
  if (!formRef.value) return
  
  try {
    await formRef.value.validate()
    submitting.value = true
    
    // 清理表单数据：将空字符串转换为undefined，确保类型正确
    const cleanFormData = (data: any) => {
      const cleaned = { ...data }
      // 确保gender是数字
      if (typeof cleaned.gender === 'string') {
        cleaned.gender = parseInt(cleaned.gender, 10)
      }
      // 处理日期字段：将Date对象转换为YYYY-MM-DD格式字符串
      const dateFields = ['birthday', 'enrollment_date', 'hire_date']
      dateFields.forEach(field => {
        if (cleaned[field] instanceof Date) {
          const date = cleaned[field] as Date
          cleaned[field] = date.toISOString().split('T')[0]
        }
      })
      // 将空字符串字段转换为undefined
      const optionalFields = ['phone', 'email', 'birthday', 'student_no', 'class_id',
        'enrollment_date', 'employee_no', 'department_id', 'title', 'hire_date',
        'wechat_openid', 'occupation']
      optionalFields.forEach(field => {
        if (cleaned[field] === '') {
          cleaned[field] = undefined
        }
      })
      // 处理密码字段：空字符串表示不修改密码（编辑时）或使用默认密码（创建时）
      if (cleaned.password === '') {
        cleaned.password = undefined
      }
      // 根据人员类型处理班级字段
      if (data.type_ === 'teacher') {
        // 教师使用classes数组，删除class_id字段
        delete cleaned.class_id
        // 清理classes数组：移除class_id为空字符串的条目
        if (cleaned.classes && Array.isArray(cleaned.classes)) {
          cleaned.classes = cleaned.classes.filter((cls: any) => cls.class_id && cls.class_id.trim() !== '')
          // 如果classes数组为空，设置为undefined
          if (cleaned.classes.length === 0) {
            cleaned.classes = undefined
          } else {
            // 如果用户标记了班主任，设置所有关联班级的班主任标志
            if (data.is_main_teacher === true) {
              cleaned.classes.forEach((cls: any) => {
                cls.is_main_teacher = true
              })
            }
          }
        }
      } else {
        // 学生或其他类型使用class_id字段，删除classes数组
        delete cleaned.classes
      }
      // 删除is_main_teacher字段，因为后端PersonUpdate结构体中没有这个字段
      delete cleaned.is_main_teacher
      // 编辑时移除type_字段，因为后端PersonUpdate结构体中没有这个字段
      if (editingId.value) {
        delete cleaned.type_
      }
      return cleaned
    }
    
    const cleanedForm = cleanFormData(form)
    
    // 打印表单数据，检查name字段
    console.log('Form data before submit:', {
      original: form,
      cleaned: cleanedForm
    })
    
    if (editingId.value) {
      // 更新
      await personApi.update(editingId.value, cleanedForm)
      ElMessage.success('更新成功')
    } else {
      // 创建
      console.log('Sending create request with data:', cleanedForm)
      await personApi.create(cleanedForm)
      ElMessage.success('创建成功')
    }
    
    dialogVisible.value = false
    loadPersons()
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
  loadPersons()
  loadClasses()
  loadDepartments()
})
</script>

<style scoped>
/* 样式已移至 ../styles/person-view.css */
</style>