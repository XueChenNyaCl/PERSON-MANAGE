<template>
  <div class="class-manage-container">
    <el-card>
      <template #header>
        <div class="card-header">
          <span>班级管理</span>
          <el-select v-model="selectedClassId" placeholder="请选择班级" @change="handleClassChange" :loading="classesLoading">
            <el-option v-for="cls in classes" :key="cls.id" :label="cls.name" :value="cls.id"></el-option>
          </el-select>
        </div>
      </template>
      
      <!-- 班级信息 -->
      <div v-if="selectedClass" class="class-info">
        <h3>{{ selectedClass.name }}</h3>
        <div class="class-detail">
          <div class="detail-item">
            <span class="label">年级：</span>
            <span>{{ selectedClass.grade }}年级</span>
          </div>
          <div class="detail-item">
            <span class="label">班主任：</span>
            <span>{{ selectedClass.teacher_name || '未设置' }}</span>
          </div>
          <div class="detail-item">
            <span class="label">学年：</span>
            <span>{{ selectedClass.academic_year }}</span>
          </div>
        </div>
      </div>
      
      <!-- 人员列表 -->
      <div v-if="selectedClassId" class="person-lists">
        <!-- 老师列表 -->
        <div class="list-section">
          <h4>班级老师</h4>
          <el-table :data="classTeachers" style="width: 100%" v-loading="teachersLoading">
            <el-table-column label="ID" width="180">
              <template #default="scope">
                <div class="id-cell">{{ scope.row.id }}</div>
              </template>
            </el-table-column>
            <el-table-column prop="name" label="姓名"></el-table-column>
            <el-table-column prop="employee_no" label="工号"></el-table-column>
            <el-table-column prop="department_name" label="部门"></el-table-column>
            <el-table-column prop="phone" label="电话" width="120"></el-table-column>
            <el-table-column label="操作" width="100">
              <template #default="scope">
                <el-button type="primary" size="small" @click="handleEditPerson(scope.row)">编辑</el-button>
              </template>
            </el-table-column>
          </el-table>
        </div>
        
        <!-- 学生列表 -->
        <div class="list-section">
          <h4>班级学生</h4>
          <el-table :data="classStudents" style="width: 100%" v-loading="studentsLoading">
            <el-table-column label="ID" width="180">
              <template #default="scope">
                <div class="id-cell">{{ scope.row.id }}</div>
              </template>
            </el-table-column>
            <el-table-column prop="name" label="姓名"></el-table-column>
            <el-table-column prop="student_no" label="学号"></el-table-column>
            <el-table-column prop="gender" label="性别" width="80">
              <template #default="scope">
                {{ scope.row.gender === 1 ? '男' : scope.row.gender === 2 ? '女' : '未知' }}
              </template>
            </el-table-column>
            <el-table-column prop="phone" label="电话" width="120"></el-table-column>
            <el-table-column label="操作" width="100">
              <template #default="scope">
                <el-button type="primary" size="small" @click="handleEditPerson(scope.row)">编辑</el-button>
              </template>
            </el-table-column>
          </el-table>
        </div>
      </div>
      
      <!-- 未选择班级提示 -->
      <div v-else class="empty-state">
        <el-empty description="请选择一个班级进行管理"></el-empty>
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { ElMessage } from 'element-plus'
import { classApi, type ClassResponse } from '../api/class'
import { personApi, type PersonResponse } from '../api/person'

// 班级相关
const classesLoading = ref(false)
const teachersLoading = ref(false)
const studentsLoading = ref(false)
const classes = ref<ClassResponse[]>([])
const selectedClassId = ref<string>('')
const selectedClass = ref<ClassResponse | null>(null)
const classTeachers = ref<PersonResponse[]>([])
const classStudents = ref<PersonResponse[]>([])

// 加载班级列表
const loadClasses = async () => {
  classesLoading.value = true
  try {
    console.log('Starting to load classes...')
    const response = await classApi.list({ page: 1, limit: 100 })
    console.log('Class API response:', response)
    console.log('Response data:', response.data)
    
    // 正确处理响应结构
    if (response && response.data) {
      classes.value = response.data.items || []
      console.log('Loaded classes:', classes.value)
      
      // 如果有班级，默认选择第一个
      if (classes.value.length > 0 && !selectedClassId.value) {
        selectedClassId.value = classes.value[0].id
        await handleClassChange()
      }
    } else {
      console.error('Invalid response structure:', response)
      classes.value = []
    }
  } catch (error) {
    ElMessage.error('加载班级列表失败')
    console.error('Error loading classes:', error)
    classes.value = []
  } finally {
    classesLoading.value = false
  }
}

// 加载班级信息
const loadClassInfo = async (classId: string) => {
  try {
    console.log('Loading class info for:', classId)
    const response = await classApi.get(classId)
    console.log('Class info response:', response)
    selectedClass.value = response.data
  } catch (error) {
    ElMessage.error('加载班级信息失败')
    console.error('Error loading class info:', error)
  }
}

// 加载班级老师
const loadClassTeachers = async (classId: string) => {
  teachersLoading.value = true
  try {
    console.log('Loading class teachers for:', classId)
    const response = await personApi.list({
      page: 1,
      limit: 100,
      type: 'teacher',
      class_id: classId
    })
    console.log('Class teachers response:', response)
    classTeachers.value = response.data.items
  } catch (error) {
    ElMessage.error('加载班级老师失败')
    console.error('Error loading class teachers:', error)
    classTeachers.value = []
  } finally {
    teachersLoading.value = false
  }
}

// 加载班级学生
const loadClassStudents = async (classId: string) => {
  studentsLoading.value = true
  try {
    console.log('Loading class students for:', classId)
    const response = await personApi.list({
      page: 1,
      limit: 100,
      type: 'student',
      class_id: classId
    })
    console.log('Class students response:', response)
    classStudents.value = response.data.items
  } catch (error) {
    ElMessage.error('加载班级学生失败')
    console.error('Error loading class students:', error)
    classStudents.value = []
  } finally {
    studentsLoading.value = false
  }
}

// 班级选择变更
const handleClassChange = async () => {
  if (selectedClassId.value) {
    await Promise.all([
      loadClassInfo(selectedClassId.value),
      loadClassTeachers(selectedClassId.value),
      loadClassStudents(selectedClassId.value)
    ])
  } else {
    selectedClass.value = null
    classTeachers.value = []
    classStudents.value = []
  }
}

// 编辑人员
const handleEditPerson = (row: PersonResponse) => {
  // 这里可以跳转到人员编辑页面或打开编辑弹窗
  console.log('Edit person:', row)
  ElMessage.info('编辑功能开发中')
}

// 初始化
onMounted(() => {
  loadClasses()
})
</script>

<style scoped>
.class-manage-container {
  padding: 20px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.class-info {
  margin-bottom: 30px;
  padding-bottom: 20px;
  border-bottom: 1px solid #e4e7ed;
}

.class-info h3 {
  margin: 0 0 15px 0;
  color: #165dff;
}

.class-detail {
  display: flex;
  gap: 30px;
}

.detail-item {
  display: flex;
  align-items: center;
}

.detail-item .label {
  font-weight: 500;
  margin-right: 5px;
}

.person-lists {
  display: flex;
  gap: 30px;
  flex-wrap: wrap;
}

.list-section {
  flex: 1;
  min-width: 400px;
}

.list-section h4 {
  margin: 0 0 15px 0;
  color: #4e5969;
}

.empty-state {
  padding: 60px 0;
  text-align: center;
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

/* 响应式设计 */
@media (max-width: 768px) {
  .person-lists {
    flex-direction: column;
  }
  
  .list-section {
    min-width: 100%;
  }
  
  .class-detail {
    flex-direction: column;
    gap: 10px;
  }
}
</style>