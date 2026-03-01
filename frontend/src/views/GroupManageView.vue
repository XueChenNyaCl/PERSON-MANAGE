<template>
  <div class="group-manage-container">
    <el-card>
      <template #header>
        <div class="card-header">
          <span>小组管理</span>
          <div class="header-actions">
            <el-select v-model="selectedClassId" placeholder="请选择班级" @change="handleClassChange" :loading="classesLoading">
              <el-option v-for="cls in classes" :key="cls.id" :label="cls.name" :value="cls.id"></el-option>
            </el-select>
            <el-button v-if="hasClassPermission('group.create', selectedClassId) && selectedClassId" type="primary" @click="showCreateDialog = true">
              创建小组
            </el-button>
          </div>
        </div>
      </template>
      
      <!-- 小组列表 -->
      <div v-if="selectedClassId" class="group-list">
        <el-row :gutter="20">
          <el-col :xs="24" :sm="12" :md="8" :lg="6" v-for="group in groups" :key="group.id">
            <el-card class="group-card" shadow="hover" @click="viewGroupDetail(group)">
              <div class="group-header">
                <h3 class="group-name">{{ group.name }}</h3>
                <el-tag :type="group.score >= 0 ? 'success' : 'danger'" class="score-tag">
                  {{ group.score }}分
                </el-tag>
              </div>
              <div class="group-info">
                <p class="description">{{ group.description || '暂无描述' }}</p>
                <div class="meta-info">
                  <span class="member-count">
                    <el-icon><User /></el-icon>
                    {{ group.member_count }}人
                  </span>
                </div>
              </div>
              <div class="group-actions" @click.stop>
                <el-button v-if="hasClassPermission('group.update', selectedClassId)" type="primary" size="small" @click="editGroup(group)">
                  编辑
                </el-button>
                <el-button v-if="hasClassPermission('group.delete', selectedClassId)" type="danger" size="small" @click="deleteGroup(group)">
                  删除
                </el-button>
              </div>
            </el-card>
          </el-col>
        </el-row>
        
        <!-- 空状态 -->
        <el-empty v-if="groups.length === 0 && !groupsLoading" description="该班级暂无小组"></el-empty>
      </div>
      
      <!-- 未选择班级提示 -->
      <div v-else class="empty-state">
        <el-empty description="请选择一个班级查看小组"></el-empty>
      </div>
    </el-card>
    
    <!-- 创建小组对话框 -->
    <el-dialog v-model="showCreateDialog" title="创建小组" width="500px">
      <el-form :model="createForm" :rules="rules" ref="createFormRef" label-width="80px">
        <el-form-item label="小组名称" prop="name">
          <el-input v-model="createForm.name" placeholder="请输入小组名称"></el-input>
        </el-form-item>
        <el-form-item label="小组描述" prop="description">
          <el-input v-model="createForm.description" type="textarea" :rows="3" placeholder="请输入小组描述"></el-input>
        </el-form-item>
      </el-form>
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="showCreateDialog = false">取消</el-button>
          <el-button type="primary" @click="submitCreate" :loading="submitLoading">确定</el-button>
        </span>
      </template>
    </el-dialog>
    
    <!-- 编辑小组对话框 -->
    <el-dialog v-model="showEditDialog" title="编辑小组" width="500px">
      <el-form :model="editForm" :rules="rules" ref="editFormRef" label-width="80px">
        <el-form-item label="小组名称" prop="name">
          <el-input v-model="editForm.name" placeholder="请输入小组名称"></el-input>
        </el-form-item>
        <el-form-item label="小组描述" prop="description">
          <el-input v-model="editForm.description" type="textarea" :rows="3" placeholder="请输入小组描述"></el-input>
        </el-form-item>
      </el-form>
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="showEditDialog = false">取消</el-button>
          <el-button type="primary" @click="submitEdit" :loading="submitLoading">确定</el-button>
        </span>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage, ElMessageBox } from 'element-plus'
import { User } from '@element-plus/icons-vue'
import { classApi, type ClassResponse } from '../api/class'
import { groupApi, type Group, type GroupCreate, type GroupUpdate } from '../api/group'
import { useAuthStore } from '../store/auth'

const router = useRouter()
const authStore = useAuthStore()

// 权限检查 - 通用权限


// 权限检查 - 班级特定权限
const hasClassPermission = (permission: string, classId: string): boolean => {
  return authStore.hasClassPermission(permission, classId)
}

// 班级相关
const classesLoading = ref(false)
const classes = ref<ClassResponse[]>([])
const selectedClassId = ref<string>('')

// 小组相关
const groupsLoading = ref(false)
const groups = ref<Group[]>([])

// 对话框相关
const showCreateDialog = ref(false)
const showEditDialog = ref(false)
const submitLoading = ref(false)
const createFormRef = ref()
const editFormRef = ref()

const createForm = reactive<GroupCreate>({
  class_id: '',
  name: '',
  description: ''
})

const editForm = reactive<GroupUpdate & { id?: string }>({
  id: '',
  name: '',
  description: ''
})

const rules = {
  name: [
    { required: true, message: '请输入小组名称', trigger: 'blur' },
    { min: 2, max: 50, message: '长度在 2 到 50 个字符', trigger: 'blur' }
  ]
}

// 加载班级列表
const loadClasses = async () => {
  classesLoading.value = true
  try {
    const response = await classApi.list({ page: 1, limit: 100 })
    if (response && response.data) {
      classes.value = response.data.items || []
      if (classes.value.length > 0 && !selectedClassId.value) {
        selectedClassId.value = classes.value[0].id
        await handleClassChange()
      }
    }
  } catch (error) {
    ElMessage.error('加载班级列表失败')
    console.error('Error loading classes:', error)
  } finally {
    classesLoading.value = false
  }
}

// 加载小组列表
const loadGroups = async () => {
  if (!selectedClassId.value) return
  
  groupsLoading.value = true
  try {
    const data = await groupApi.getGroupsByClass(selectedClassId.value)
    groups.value = data
  } catch (error) {
    ElMessage.error('加载小组列表失败')
    console.error('Error loading groups:', error)
  } finally {
    groupsLoading.value = false
  }
}

// 班级选择变更
const handleClassChange = async () => {
  createForm.class_id = selectedClassId.value
  await loadGroups()
}

// 查看小组详情
const viewGroupDetail = (group: Group) => {
  router.push(`/dashboard/group/${group.id}`)
}

// 创建小组
const submitCreate = async () => {
  if (!createFormRef.value) return
  
  await createFormRef.value.validate(async (valid: boolean) => {
    if (!valid) return
    
    submitLoading.value = true
    try {
      await groupApi.createGroup(createForm)
      ElMessage.success('创建小组成功')
      showCreateDialog.value = false
      createForm.name = ''
      createForm.description = ''
      await loadGroups()
    } catch (error: any) {
      ElMessage.error(error.response?.data?.message || '创建小组失败')
    } finally {
      submitLoading.value = false
    }
  })
}

// 编辑小组
const editGroup = (group: Group) => {
  editForm.id = group.id
  editForm.name = group.name
  editForm.description = group.description || ''
  showEditDialog.value = true
}

const submitEdit = async () => {
  if (!editFormRef.value || !editForm.id) return
  
  await editFormRef.value.validate(async (valid: boolean) => {
    if (!valid) return
    
    submitLoading.value = true
    try {
      if (editForm.id) {
        await groupApi.updateGroup(editForm.id, {
          name: editForm.name,
          description: editForm.description
        })
        ElMessage.success('更新小组成功')
      }
      showEditDialog.value = false
      await loadGroups()
    } catch (error: any) {
      ElMessage.error(error.response?.data?.message || '更新小组失败')
    } finally {
      submitLoading.value = false
    }
  })
}

// 删除小组
const deleteGroup = async (group: Group) => {
  try {
    await ElMessageBox.confirm(
      `确定要删除小组 "${group.name}" 吗？`,
      '确认删除',
      {
        confirmButtonText: '确定',
        cancelButtonText: '取消',
        type: 'warning'
      }
    )
    
    await groupApi.deleteGroup(group.id)
    ElMessage.success('删除小组成功')
    await loadGroups()
  } catch (error: any) {
    if (error !== 'cancel') {
      ElMessage.error(error.response?.data?.message || '删除小组失败')
    }
  }
}

// 初始化
onMounted(() => {
  loadClasses()
})
</script>

<style scoped>
.group-manage-container {
  padding: 20px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.header-actions {
  display: flex;
  gap: 10px;
  align-items: center;
}

.group-list {
  margin-top: 20px;
}

.group-card {
  margin-bottom: 20px;
  cursor: pointer;
  transition: all 0.3s;
}

.group-card:hover {
  transform: translateY(-5px);
}

.group-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 10px;
}

.group-name {
  margin: 0;
  font-size: 16px;
  color: #303133;
}

.score-tag {
  font-size: 14px;
  font-weight: bold;
}

.group-info {
  margin-bottom: 15px;
}

.description {
  margin: 0 0 10px 0;
  color: #606266;
  font-size: 14px;
  min-height: 40px;
  overflow: hidden;
  text-overflow: ellipsis;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
}

.meta-info {
  display: flex;
  gap: 15px;
  color: #909399;
  font-size: 13px;
}

.member-count {
  display: flex;
  align-items: center;
  gap: 4px;
}

.group-actions {
  display: flex;
  gap: 8px;
  justify-content: flex-end;
  padding-top: 10px;
  border-top: 1px solid #ebeef5;
}

.empty-state {
  padding: 60px 0;
  text-align: center;
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
}

:deep(.el-card__header) {
  padding: 15px 20px;
}

:deep(.el-card__body) {
  padding: 15px;
}
</style>
