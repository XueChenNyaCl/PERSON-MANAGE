<template>
  <div class="special-user-tab">
    <div class="tab-header">
      <h3>特殊用户管理</h3>
      <el-button type="primary" @click="handleAdd" v-if="canCreate">
        <el-icon><Plus /></el-icon>
        新增特殊用户
      </el-button>
    </div>

    <el-alert
      title="特殊用户说明"
      type="info"
      :closable="false"
      class="info-alert"
    >
      <template #default>
        <div class="info-content">
          <p><strong>System:</strong> 系统内部操作用户，不可登录</p>
          <p><strong>SysAI:</strong> 系统AI用户，暂留功能</p>
          <p><strong>ChatAI:</strong> 聊天AI操作记录用户</p>
          <p><strong>IoT:</strong> 物联网设备用户，可通过API密钥登录</p>
          <p><strong>Scerm:</strong> 大屏展示用户，可通过API密钥登录</p>
        </div>
      </template>
    </el-alert>

    <el-table :data="specialUsers" v-loading="loading" style="width: 100%">
      <el-table-column prop="identifier" label="标识符" width="180" />
      <el-table-column prop="user_type" label="类型" width="120">
        <template #default="scope">
          <el-tag :type="getTypeTag(scope.row.user_type)">
            {{ getTypeLabel(scope.row.user_type) }}
          </el-tag>
        </template>
      </el-table-column>
      <el-table-column prop="description" label="描述" show-overflow-tooltip />
      <el-table-column prop="linked_person_name" label="关联人员" width="150">
        <template #default="scope">
          <span v-if="scope.row.linked_person_name">{{ scope.row.linked_person_name }}</span>
          <el-tag v-else type="info" size="small">未关联</el-tag>
        </template>
      </el-table-column>
      <el-table-column prop="is_active" label="状态" width="100">
        <template #default="scope">
          <el-tag :type="scope.row.is_active ? 'success' : 'danger'" size="small">
            {{ scope.row.is_active ? '激活' : '禁用' }}
          </el-tag>
        </template>
      </el-table-column>
      <el-table-column prop="last_login_at" label="最后登录" width="180">
        <template #default="scope">
          {{ scope.row.last_login_at ? formatDate(scope.row.last_login_at) : '从未登录' }}
        </template>
      </el-table-column>
      <el-table-column label="操作" width="200" fixed="right">
        <template #default="scope">
          <el-button
            v-if="canLink && !isSystemType(scope.row.user_type)"
            type="primary"
            size="small"
            @click="handleLink(scope.row)"
          >
            关联
          </el-button>
          <el-button
            v-if="canDelete && !isSystemType(scope.row.user_type)"
            type="danger"
            size="small"
            @click="handleDelete(scope.row)"
          >
            删除
          </el-button>
        </template>
      </el-table-column>
    </el-table>

    <!-- 新增特殊用户对话框 -->
    <el-dialog
      v-model="dialogVisible"
      title="新增特殊用户"
      width="500px"
    >
      <el-form :model="form" :rules="rules" ref="formRef" label-width="100px">
        <el-form-item label="用户类型" prop="user_type">
          <el-select v-model="form.user_type" placeholder="请选择用户类型" style="width: 100%">
            <el-option label="物联网设备 (IoT)" value="iot" />
            <el-option label="大屏展示 (Scerm)" value="scerm" />
          </el-select>
        </el-form-item>
        <el-form-item label="标识符" prop="identifier">
          <el-input v-model="form.identifier" placeholder="如: device001, screen01">
            <template #prepend v-if="form.user_type">
              {{ form.user_type }}:
            </template>
          </el-input>
        </el-form-item>
        <el-form-item label="API密钥" prop="api_key">
          <el-input
            v-model="form.api_key"
            type="password"
            placeholder="用于设备登录的API密钥"
            show-password
          />
        </el-form-item>
        <el-form-item label="描述" prop="description">
          <el-input
            v-model="form.description"
            type="textarea"
            :rows="3"
            placeholder="可选：描述该特殊用户的用途"
          />
        </el-form-item>
      </el-form>
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="dialogVisible = false">取消</el-button>
          <el-button type="primary" @click="handleSubmit" :loading="submitting">
            确定
          </el-button>
        </span>
      </template>
    </el-dialog>

    <!-- 关联人员对话框 -->
    <el-dialog
      v-model="linkDialogVisible"
      title="关联人员"
      width="400px"
    >
      <el-form :model="linkForm" ref="linkFormRef" label-width="100px">
        <el-form-item label="选择人员" prop="person_id">
          <el-select
            v-model="linkForm.person_id"
            filterable
            remote
            reserve-keyword
            placeholder="请输入人员姓名搜索"
            :remote-method="searchPersons"
            :loading="personLoading"
            style="width: 100%"
          >
            <el-option
              v-for="person in personOptions"
              :key="person.id"
              :label="person.name"
              :value="person.id"
            >
              <span>{{ person.name }}</span>
              <span style="float: right; color: #8492a6; font-size: 13px">
                {{ person.type === 'student' ? '学生' : person.type === 'teacher' ? '教师' : '家长' }}
              </span>
            </el-option>
          </el-select>
        </el-form-item>
      </el-form>
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="linkDialogVisible = false">取消</el-button>
          <el-button type="primary" @click="handleLinkSubmit" :loading="linkSubmitting">
            确定
          </el-button>
        </span>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { Plus } from '@element-plus/icons-vue'
import type { FormInstance } from 'element-plus'
import { specialUserApi, type SpecialUserResponse, type CreateSpecialUserRequest } from '../api/specialUser'
import { personApi, type PersonResponse } from '../api/person'
import { useAuthStore } from '../store/auth'

const authStore = useAuthStore()

// 权限检查
const canCreate = computed(() => authStore.hasPermission('special_user.create'))
const canDelete = computed(() => authStore.hasPermission('special_user.delete'))
const canLink = computed(() => authStore.hasPermission('special_user.link'))

// 数据
const loading = ref(false)
const specialUsers = ref<SpecialUserResponse[]>([])
const dialogVisible = ref(false)
const submitting = ref(false)
const formRef = ref<FormInstance>()

const form = ref<CreateSpecialUserRequest>({
  user_type: 'iot',
  identifier: '',
  api_key: '',
  description: '',
})

const rules = {
  user_type: [{ required: true, message: '请选择用户类型', trigger: 'change' }],
  identifier: [{ required: true, message: '请输入标识符', trigger: 'blur' }],
  api_key: [{ required: true, message: '请输入API密钥', trigger: 'blur' }],
}

// 关联人员
const linkDialogVisible = ref(false)
const linkSubmitting = ref(false)
const linkFormRef = ref<FormInstance>()
const currentSpecialUser = ref<SpecialUserResponse | null>(null)
const linkForm = ref({ person_id: '' })
const personOptions = ref<PersonResponse[]>([])
const personLoading = ref(false)

// 加载特殊用户列表
const loadSpecialUsers = async () => {
  loading.value = true
  try {
    const response = await specialUserApi.list()
    specialUsers.value = response.data
  } catch (error) {
    ElMessage.error('加载特殊用户列表失败')
    console.error('Error loading special users:', error)
  } finally {
    loading.value = false
  }
}

// 获取类型标签
const getTypeTag = (type: string) => {
  const tagMap: Record<string, string> = {
    system: 'danger',
    sysai: 'warning',
    chatai: 'success',
    iot: 'primary',
    scerm: 'info',
  }
  return tagMap[type] || ''
}

// 获取类型标签文本
const getTypeLabel = (type: string) => {
  const labelMap: Record<string, string> = {
    system: 'System',
    sysai: 'SysAI',
    chatai: 'ChatAI',
    iot: 'IoT',
    scerm: 'Scerm',
  }
  return labelMap[type] || type
}

// 检查是否是系统保留类型
const isSystemType = (type: string) => {
  return ['system', 'sysai', 'chatai'].includes(type)
}

// 格式化日期
const formatDate = (date: string) => {
  return new Date(date).toLocaleString('zh-CN')
}

// 新增
const handleAdd = () => {
  form.value = {
    user_type: 'iot',
    identifier: '',
    api_key: '',
    description: '',
  }
  dialogVisible.value = true
}

// 提交新增
const handleSubmit = async () => {
  if (!formRef.value) return

  try {
    await formRef.value.validate()
    submitting.value = true

    await specialUserApi.create(form.value)
    ElMessage.success('创建成功')
    dialogVisible.value = false
    loadSpecialUsers()
  } catch (error) {
    ElMessage.error('创建失败')
    console.error('Error creating special user:', error)
  } finally {
    submitting.value = false
  }
}

// 删除
const handleDelete = async (row: SpecialUserResponse) => {
  try {
    await ElMessageBox.confirm(
      `确定要删除特殊用户 "${row.identifier}" 吗？`,
      '警告',
      {
        confirmButtonText: '确定',
        cancelButtonText: '取消',
        type: 'warning',
      }
    )

    await specialUserApi.delete(row.id)
    ElMessage.success('删除成功')
    loadSpecialUsers()
  } catch (error) {
    if (error !== 'cancel') {
      ElMessage.error('删除失败')
      console.error('Error deleting special user:', error)
    }
  }
}

// 关联人员
const handleLink = (row: SpecialUserResponse) => {
  currentSpecialUser.value = row
  linkForm.value.person_id = ''
  linkDialogVisible.value = true
}

// 搜索人员
const searchPersons = async (query: string) => {
  if (query.length < 1) return

  personLoading.value = true
  try {
    const response = await personApi.list({
      search: query,
      limit: 20,
    })
    personOptions.value = response.data.items
  } catch (error) {
    console.error('Error searching persons:', error)
  } finally {
    personLoading.value = false
  }
}

// 提交关联
const handleLinkSubmit = async () => {
  if (!currentSpecialUser.value || !linkForm.value.person_id) {
    ElMessage.warning('请选择要关联的人员')
    return
  }

  linkSubmitting.value = true
  try {
    await specialUserApi.linkPerson(currentSpecialUser.value.id, {
      person_id: linkForm.value.person_id,
    })
    ElMessage.success('关联成功')
    linkDialogVisible.value = false
    loadSpecialUsers()
  } catch (error) {
    ElMessage.error('关联失败')
    console.error('Error linking person:', error)
  } finally {
    linkSubmitting.value = false
  }
}

onMounted(() => {
  loadSpecialUsers()
})
</script>

<style scoped>
.special-user-tab {
  padding: 20px;
}

.tab-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.tab-header h3 {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
}

.info-alert {
  margin-bottom: 20px;
}

.info-content {
  font-size: 13px;
  line-height: 1.8;
}

.info-content p {
  margin: 4px 0;
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
}
</style>
