<template>
  <div class="login-container">
    <div class="login-form">
      <h1>智联校园多维管理平台</h1>
      <el-form :model="loginForm" :rules="rules" ref="loginFormRef" label-width="80px">
        <el-form-item label="用户名" prop="username">
          <el-input v-model="loginForm.username" placeholder="请输入用户名"></el-input>
        </el-form-item>
        <el-form-item label="密码" prop="password">
          <el-input v-model="loginForm.password" type="password" placeholder="请输入密码"></el-input>
        </el-form-item>
        <el-form-item>
          <el-checkbox v-model="loginForm.rememberMe">记住我</el-checkbox>
        </el-form-item>
        <el-form-item>
          <el-button type="primary" @click="handleLogin" :loading="loading" style="width: 100%">登录</el-button>
        </el-form-item>
      </el-form>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import type { FormInstance, FormRules } from 'element-plus'
import { ElMessage } from 'element-plus'
import { authApi } from '../api/auth'
import { useAuthStore } from '../store/auth'
import { useMobile } from '../composables/useMobile'

const router = useRouter()
const loginFormRef = ref<FormInstance>()
const loading = ref(false)
const authStore = useAuthStore()

// 移动端适配
const { isMobile, loadMobileStyle } = useMobile()

// 动态加载移动端样式
onMounted(async () => {
  if (isMobile.value) {
    await loadMobileStyle('login-mobile')
  }
})

const loginForm = reactive({
  username: '',
  password: '',
  rememberMe: false
})

const rules = reactive<FormRules>({
  username: [
    { required: true, message: '请输入用户名', trigger: 'blur' }
  ],
  password: [
    { required: true, message: '请输入密码', trigger: 'blur' }
  ]
})

const handleLogin = async () => {
  if (!loginFormRef.value) return
  
  try {
    await loginFormRef.value.validate()
    loading.value = true
    
    // 调用真实API登录
    const response = await authApi.login({
      username: loginForm.username,
      password: loginForm.password,
      remember_me: loginForm.rememberMe
    })
    
    // 保存令牌和用户信息到store和localStorage
    authStore.setAuth(response.data.token, response.data.user, response.data.permissions, response.data.class_permissions)
    
    ElMessage.success('登录成功')
    router.push('/dashboard')
  } catch (error: any) {
    console.error('登录失败:', error)
    ElMessage.error(error.response?.data?.message || error.message || '登录失败，请检查用户名和密码')
  } finally {
    loading.value = false
  }
}
</script>

<style scoped src="@/styles/login-view.css"></style>