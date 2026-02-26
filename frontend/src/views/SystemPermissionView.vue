<template>
  <div class="permission-management-container">
    <!-- 页面标题 -->
    <div class="page-header">
      <h1>{{ $t('user_permission_page.title') }}</h1>
      <div class="header-actions">
        <el-button type="primary" @click="handleImportYaml">
          <el-icon><Upload /></el-icon>
          {{ $t('permission.import_yaml') }}
        </el-button>
        <el-button @click="refreshData">
          <el-icon><Refresh /></el-icon>
          {{ $t('common.refresh') }}
        </el-button>
      </div>
    </div>

    <!-- 主内容区域 -->
    <div class="main-content">
      <!-- 左侧用户列表 -->
      <div class="left-panel">
        <div class="panel-header">
          <h3>{{ $t('user_permission_page.user_list') }}</h3>
          <el-input
            v-model="userSearch"
            :placeholder="$t('user_permission_page.search_user')"
            clearable
            @clear="clearUserSearch"
          >
            <template #prefix>
              <el-icon><Search /></el-icon>
            </template>
          </el-input>
        </div>
        <div class="panel-body">
          <el-table
            :data="filteredUsers"
            highlight-current-row
            @row-click="handleUserSelect"
            style="width: 100%"
          >
            <el-table-column prop="name" :label="$t('common.username')" width="120" />
            <el-table-column prop="type" :label="$t('common.role')" width="100">
              <template #default="{ row }">
                {{ $t(`roles.${row.type}`) }}
              </template>
            </el-table-column>
            <el-table-column prop="email" :label="$t('common.email')" />
            <el-table-column :label="$t('common.operation')" width="80">
              <template #default="{ row }">
                <el-button
                  type="text"
                  size="small"
                  @click.stop="viewUserPermissions(row)"
                >
                  {{ $t('permission.check_permission') }}
                </el-button>
              </template>
            </el-table-column>
          </el-table>
        </div>
      </div>

      <!-- 中间权限操作区 -->
      <div class="center-panel">
        <div class="panel-header">
          <h3 v-if="selectedUser">
            {{ $t('user_permission_page.add_permission_to_user') }}: {{ selectedUser.name }}
          </h3>
          <h3 v-else>
            {{ $t('user_permission_page.no_user_selected') }}
          </h3>
        </div>
        <div class="panel-body">
          <!-- 权限搜索和添加 -->
          <div class="permission-search-section">
            <el-input
              v-model="permissionSearch"
              :placeholder="$t('user_permission_page.search_permission')"
              clearable
              style="margin-bottom: 16px"
            >
              <template #prefix>
                <el-icon><Search /></el-icon>
              </template>
            </el-input>
            
            <div class="permission-actions">
              <el-input
                v-model="newPermissionKey"
                :placeholder="$t('permission.permission_key')"
                style="width: 300px; margin-right: 12px"
              />
              <el-input-number
                v-model="newPermissionPriority"
                :min="1"
                :max="100"
                :placeholder="$t('permission.priority')"
                style="width: 120px; margin-right: 12px"
              />
              <el-button
                type="primary"
                :disabled="!selectedUser || !newPermissionKey"
                @click="addPermissionToUser"
              >
                {{ $t('user_permission_page.add') }}
              </el-button>
              <el-button
                type="danger"
                :disabled="!selectedPermission"
                @click="removePermissionFromUser"
              >
                {{ $t('user_permission_page.remove') }}
              </el-button>
            </div>
          </div>

          <!-- 用户当前权限列表 -->
          <div class="permission-list-section">
            <h4>{{ $t('user_permission_page.permission_list') }}</h4>
            <el-table
              :data="filteredUserPermissions"
              highlight-current-row
              @row-click="handlePermissionSelect"
              style="width: 100%"
            >
              <el-table-column prop="permission" :label="$t('permission.permission_key')">
                <template #default="{ row }">
                  <span :class="{ 'negation-permission': row.permission.startsWith('-') }">
                    {{ row.permission }}
                  </span>
                </template>
              </el-table-column>
              <el-table-column :label="$t('permission.permission_name')" width="180">
                <template #default="{ row }">
                  {{ $t(`permissions.${row.permission.replace(/^-+/, '')}`) }}
                </template>
              </el-table-column>
              <el-table-column prop="priority" :label="$t('permission.priority')" width="100" />
              <el-table-column :label="$t('user_permission_page.effective')" width="100">
                <template #default="{ row }">
                  <el-tag v-if="isPermissionEffective(row)" type="success">有效</el-tag>
                  <el-tag v-else type="danger">无效</el-tag>
                </template>
              </el-table-column>
              <el-table-column :label="$t('common.operation')" width="80">
                <template #default="{ row }">
                  <el-button
                    type="text"
                    size="small"
                    @click.stop="removePermission(row)"
                  >
                    {{ $t('common.delete') }}
                  </el-button>
                </template>
              </el-table-column>
            </el-table>
          </div>
        </div>
      </div>

      <!-- 右侧批量操作区 -->
      <div class="right-panel">
        <div class="panel-header">
          <h3>{{ $t('user_permission_page.bulk_operations') }}</h3>
        </div>
        <div class="panel-body">
          <!-- YAML导入 -->
          <div class="bulk-operation-section">
            <h4>{{ $t('yaml_import.title') }}</h4>
            <p class="description">{{ $t('yaml_import.description') }}</p>
            
            <el-upload
              class="yaml-upload"
              :auto-upload="false"
              :show-file-list="false"
              :on-change="handleYamlFileChange"
              accept=".yaml,.yml"
            >
              <el-button type="primary">
                <el-icon><Upload /></el-icon>
                {{ $t('yaml_import.select_file') }}
              </el-button>
            </el-upload>
            
            <div v-if="yamlContent" class="yaml-preview">
              <h5>{{ $t('yaml_import.template_preview') }}</h5>
              <pre>{{ yamlContent }}</pre>
            </div>
            
            <div class="upload-actions">
              <el-select
                v-model="yamlApplyTarget"
                :placeholder="$t('yaml_import.apply_to_role')"
                style="width: 200px; margin-right: 12px"
              >
                <el-option label="应用到选中用户" value="user" />
                <el-option label="应用到角色" value="role" />
                <el-option label="应用到所有用户" value="all" />
              </el-select>
              
              <el-select
                v-model="yamlMergeStrategy"
                :placeholder="$t('yaml_import.merge_with_existing')"
                style="width: 200px; margin-right: 12px"
              >
                <el-option label="覆盖现有权限" value="overwrite" />
                <el-option label="合并现有权限" value="merge" />
              </el-select>
              
              <el-button
                type="primary"
                :disabled="!yamlContent || !yamlApplyTarget"
                @click="applyYamlTemplate"
              >
                {{ $t('yaml_import.apply_to_user') }}
              </el-button>
            </div>
          </div>

          <!-- 批量权限操作 -->
          <div class="bulk-permission-section">
            <h4>{{ $t('permission.bulk_update') }}</h4>
            
            <div class="bulk-inputs">
              <el-input
                v-model="bulkPermissionKey"
                :placeholder="$t('permission.permission_key')"
                style="margin-bottom: 12px"
              />
              <el-input-number
                v-model="bulkPermissionPriority"
                :min="1"
                :max="100"
                :placeholder="$t('permission.priority')"
                style="width: 100%; margin-bottom: 12px"
              />
            </div>
            
            <div class="bulk-actions">
              <el-button
                type="primary"
                :disabled="!selectedUser || !bulkPermissionKey"
                @click="addBulkPermission"
              >
                {{ $t('permission.add_permission') }}
              </el-button>
              <el-button
                type="danger"
                :disabled="!selectedUser || !bulkPermissionKey"
                @click="removeBulkPermission"
              >
                {{ $t('permission.remove_permission') }}
              </el-button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- YAML导入对话框 -->
    <el-dialog
      v-model="showYamlImportDialog"
      :title="$t('yaml_import.title')"
      width="800"
    >
      <div class="yaml-import-dialog">
        <div class="yaml-upload-area">
          <el-upload
            drag
            :auto-upload="false"
            :show-file-list="false"
            :on-change="handleYamlFileChange"
            accept=".yaml,.yml"
          >
            <el-icon class="upload-icon"><UploadFilled /></el-icon>
            <div class="upload-text">
              {{ $t('yaml_import.select_file') }}
            </div>
          </el-upload>
        </div>
        
        <div v-if="yamlContent" class="yaml-preview-area">
          <h4>{{ $t('yaml_import.template_preview') }}</h4>
          <pre>{{ yamlContent }}</pre>
        </div>
        
        <div class="import-options">
          <el-form label-width="120px">
            <el-form-item :label="$t('yaml_import.apply_to_role')">
              <el-select v-model="yamlApplyTarget" style="width: 300px">
                <el-option label="应用到选中用户" value="user" />
                <el-option label="应用到角色" value="role" />
                <el-option label="应用到所有用户" value="all" />
              </el-select>
            </el-form-item>
            <el-form-item :label="$t('yaml_import.merge_with_existing')">
              <el-select v-model="yamlMergeStrategy" style="width: 300px">
                <el-option label="覆盖现有权限" value="overwrite" />
                <el-option label="合并现有权限" value="merge" />
              </el-select>
            </el-form-item>
          </el-form>
        </div>
      </div>
      
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="showYamlImportDialog = false">
            {{ $t('common.cancel') }}
          </el-button>
          <el-button
            type="primary"
            :disabled="!yamlContent"
            @click="applyYamlTemplate"
          >
            {{ $t('yaml_import.upload') }}
          </el-button>
        </span>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { ElMessage } from 'element-plus'
import {
  Upload,
  Refresh,
  Search,
  UploadFilled
} from '@element-plus/icons-vue'

import { permissionApi } from '../api/permission'
import { personApi } from '../api/person'
import type { PersonResponse } from '../api/person'
import type { Permission } from '../api/permission'

// 导入样式文件
import '../styles/permission-management.css'

// 状态定义
const userSearch = ref('')
const permissionSearch = ref('')
const newPermissionKey = ref('')
const newPermissionPriority = ref(10)
const bulkPermissionKey = ref('')
const bulkPermissionPriority = ref(10)
const yamlContent = ref('')
const yamlApplyTarget = ref('user')
const yamlMergeStrategy = ref('merge')
const showYamlImportDialog = ref(false)

// 数据
const users = ref<PersonResponse[]>([])
const selectedUser = ref<PersonResponse | null>(null)
const selectedPermission = ref<Permission | null>(null)
const userPermissions = ref<Permission[]>([])

// 计算属性
const filteredUsers = computed(() => {
  if (!userSearch.value) return users.value
  const search = userSearch.value.toLowerCase()
  return users.value.filter(user =>
    user.name.toLowerCase().includes(search) ||
    user.email?.toLowerCase().includes(search) ||
    user.type.toLowerCase().includes(search)
  )
})

const filteredUserPermissions = computed(() => {
  if (!permissionSearch.value) return userPermissions.value
  const search = permissionSearch.value.toLowerCase()
  return userPermissions.value.filter(perm =>
    perm.permission.toLowerCase().includes(search)
  )
})

// 方法
const refreshData = async () => {
  try {
    // 加载用户列表（使用较大的限制以确保获取所有用户）
    const response = await personApi.list({ page: 1, limit: 10000 })
    users.value = response.data.items
    
    if (selectedUser.value) {
      // 重新加载选中用户的权限
      await loadUserPermissions(selectedUser.value.id)
    }
    
    ElMessage.success('数据刷新成功')
  } catch (error) {
    console.error('刷新数据失败:', error)
    ElMessage.error('刷新数据失败')
  }
}

const loadUserPermissions = async (userId: string) => {
  try {
    const permissions = await permissionApi.getUserPermissions(userId)
    userPermissions.value = permissions
  } catch (error) {
    console.error('加载用户权限失败:', error)
    ElMessage.error('加载用户权限失败')
  }
}

const handleUserSelect = (user: PersonResponse) => {
  selectedUser.value = user
  selectedPermission.value = null
  loadUserPermissions(user.id)
}

const handlePermissionSelect = (permission: Permission) => {
  selectedPermission.value = permission
}

const addPermissionToUser = async () => {
  if (!selectedUser.value || !newPermissionKey.value) return
  
  try {
    await permissionApi.addUserPermission(
      selectedUser.value.id,
      newPermissionKey.value,
      newPermissionPriority.value
    )
    
    ElMessage.success('权限添加成功')
    await loadUserPermissions(selectedUser.value.id)
    newPermissionKey.value = ''
    newPermissionPriority.value = 10
  } catch (error) {
    console.error('添加权限失败:', error)
    ElMessage.error('添加权限失败')
  }
}

const removePermissionFromUser = async () => {
  if (!selectedUser.value || !selectedPermission.value) return
  
  try {
    await permissionApi.removeUserPermission(
      selectedUser.value.id,
      selectedPermission.value.permission
    )
    
    ElMessage.success('权限移除成功')
    await loadUserPermissions(selectedUser.value.id)
    selectedPermission.value = null
  } catch (error) {
    console.error('移除权限失败:', error)
    ElMessage.error('移除权限失败')
  }
}

const removePermission = async (permission: Permission) => {
  if (!selectedUser.value) return
  
  try {
    await permissionApi.removeUserPermission(
      selectedUser.value.id,
      permission.permission
    )
    
    ElMessage.success('权限删除成功')
    await loadUserPermissions(selectedUser.value.id)
  } catch (error) {
    console.error('删除权限失败:', error)
    ElMessage.error('删除权限失败')
  }
}

const viewUserPermissions = (user: PersonResponse) => {
  selectedUser.value = user
  loadUserPermissions(user.id)
}

const isPermissionEffective = (_permission: Permission) => {
  // 这里需要调用后端API检查权限是否有效
  // 暂时返回true
  return true
}

// YAML导入相关方法
const handleImportYaml = () => {
  showYamlImportDialog.value = true
}

const handleYamlFileChange = (file: any) => {
  const reader = new FileReader()
  reader.onload = (e) => {
    yamlContent.value = e.target?.result as string
  }
  reader.readAsText(file.raw)
}

const applyYamlTemplate = async () => {
  if (!yamlContent.value) return
  
  try {
    let targetType: 'user' | 'role' | 'all' = 'user'
    let targetIds: string[] = []
    let role: string | undefined = undefined
    
    if (yamlApplyTarget.value === 'user' && selectedUser.value) {
      targetType = 'user'
      targetIds = [selectedUser.value.id]
    } else if (yamlApplyTarget.value === 'role' && selectedUser.value) {
      targetType = 'role'
      role = selectedUser.value.type
    } else if (yamlApplyTarget.value === 'all') {
      targetType = 'all'
    }
    
    // 调用后端API应用YAML模板（批量）
    console.log('=== YAML IMPORT DEBUG ===')
    console.log('yamlContent:', yamlContent.value)
    console.log('targetType:', targetType)
    console.log('targetIds:', targetIds)
    console.log('role:', role)
    console.log('merge_strategy:', yamlMergeStrategy.value)
    
    // 验证目标选择
    if (targetType === 'user' && targetIds.length === 0) {
      ElMessage.error('请选择要应用权限的用户')
      return
    }
    if (targetType === 'role' && !role) {
      ElMessage.error('请选择要应用权限的角色')
      return
    }
    
    const request = {
      yaml_content: yamlContent.value,
      target_type: targetType,
      target_ids: targetIds.length > 0 ? targetIds : undefined,
      role,
      merge_strategy: yamlMergeStrategy.value as 'overwrite' | 'merge'
    }
    
    console.log('Request payload:', JSON.stringify(request, null, 2))
    const response = await permissionApi.applyYamlTemplateBulk(request)
    
    if (response.data.success) {
      ElMessage.success(`YAML模板应用成功，已应用到 ${response.data.applied_count} 个目标`)
    } else {
      ElMessage.warning(`YAML模板应用完成，但部分失败: ${response.data.message}`)
    }
    
    showYamlImportDialog.value = false
    yamlContent.value = ''
    
    if (selectedUser.value) {
      await loadUserPermissions(selectedUser.value.id)
    }
  } catch (error) {
    console.error('应用YAML模板失败:', error)
    ElMessage.error('应用YAML模板失败')
  }
}

const addBulkPermission = async () => {
  if (!selectedUser.value || !bulkPermissionKey.value) return
  
  try {
    await permissionApi.addUserPermission(
      selectedUser.value.id,
      bulkPermissionKey.value,
      bulkPermissionPriority.value
    )
    
    ElMessage.success('批量权限添加成功')
    await loadUserPermissions(selectedUser.value.id)
    bulkPermissionKey.value = ''
    bulkPermissionPriority.value = 10
  } catch (error) {
    console.error('批量添加权限失败:', error)
    ElMessage.error('批量添加权限失败')
  }
}

const removeBulkPermission = async () => {
  if (!selectedUser.value || !bulkPermissionKey.value) return
  
  try {
    await permissionApi.removeUserPermission(
      selectedUser.value.id,
      bulkPermissionKey.value
    )
    
    ElMessage.success('批量权限移除成功')
    await loadUserPermissions(selectedUser.value.id)
    bulkPermissionKey.value = ''
  } catch (error) {
    console.error('批量移除权限失败:', error)
    ElMessage.error('批量移除权限失败')
  }
}

const clearUserSearch = () => {
  userSearch.value = ''
}

// 生命周期
onMounted(async () => {
  await refreshData()
})
</script>

<style scoped>
/* 基础样式已经导入外部CSS文件，这里只写组件特定的样式 */
.negation-permission {
  color: #f56c6c;
  font-weight: bold;
}

.permission-management-container {
  height: 100%;
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
}

.main-content {
  display: flex;
  gap: 20px;
  height: calc(100vh - 120px);
}

.left-panel,
.center-panel,
.right-panel {
  background: #fff;
  border-radius: 8px;
  padding: 20px;
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.08);
}

.left-panel {
  flex: 1;
}

.center-panel {
  flex: 2;
}

.right-panel {
  flex: 1;
}

.panel-header {
  margin-bottom: 20px;
}

.panel-body {
  height: calc(100% - 60px);
  overflow-y: auto;
}

.permission-search-section {
  margin-bottom: 24px;
}

.permission-actions {
  display: flex;
  align-items: center;
  gap: 12px;
}

.permission-list-section {
  margin-top: 24px;
}

.bulk-operation-section,
.bulk-permission-section {
  margin-bottom: 24px;
}

.description {
  color: #666;
  font-size: 14px;
  margin-bottom: 16px;
}

.yaml-upload {
  margin-bottom: 16px;
}

.yaml-preview {
  background: #f5f5f5;
  padding: 16px;
  border-radius: 4px;
  margin-bottom: 16px;
  max-height: 200px;
  overflow-y: auto;
}

.yaml-preview pre {
  margin: 0;
  white-space: pre-wrap;
  word-wrap: break-word;
}

.upload-actions {
  display: flex;
  align-items: center;
  gap: 12px;
}

.bulk-inputs {
  margin-bottom: 16px;
}

.bulk-actions {
  display: flex;
  gap: 12px;
}

.yaml-import-dialog {
  padding: 20px;
}

.upload-icon {
  font-size: 48px;
  color: #409eff;
}

.upload-text {
  margin-top: 12px;
  color: #666;
}

.yaml-preview-area {
  margin-top: 24px;
}

.yaml-preview-area pre {
  background: #f5f5f5;
  padding: 16px;
  border-radius: 4px;
  max-height: 300px;
  overflow-y: auto;
}

.import-options {
  margin-top: 24px;
}
</style>