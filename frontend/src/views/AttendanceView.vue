<template>
  <div class="attendance-container">
    <el-card>
      <template #header>
        <div class="card-header">
          <span>考勤管理</span>
          <el-button type="primary" @click="handleAdd">新增考勤</el-button>
        </div>
      </template>
      <el-form :inline="true" :model="searchForm" class="search-form">
        <el-form-item label="姓名">
          <el-input v-model="searchForm.name" placeholder="请输入姓名"></el-input>
        </el-form-item>
        <el-form-item label="日期">
          <el-date-picker v-model="searchForm.date" type="date" placeholder="选择日期"></el-date-picker>
        </el-form-item>
        <el-form-item>
          <el-button type="primary" @click="handleSearch">搜索</el-button>
        </el-form-item>
      </el-form>
      <el-table :data="attendanceList" style="width: 100%">
        <el-table-column prop="id" label="ID" width="80"></el-table-column>
        <el-table-column prop="name" label="姓名"></el-table-column>
        <el-table-column prop="date" label="日期" width="120"></el-table-column>
        <el-table-column prop="status" label="状态">
          <template #default="scope">
            <el-tag :type="getTagType(scope.row.status)">{{ scope.row.status }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="time" label="时间" width="180"></el-table-column>
        <el-table-column prop="remark" label="备注"></el-table-column>
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
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'

interface Attendance {
  id: number
  name: string
  date: string
  status: '正常' | '迟到' | '缺勤' | '早退'
  time: string
  remark: string
}

const attendanceList = ref<Attendance[]>([
  { id: 1, name: '张三', date: '2023-09-01', status: '正常', time: '08:00:00', remark: '' },
  { id: 2, name: '李四', date: '2023-09-01', status: '迟到', time: '08:15:00', remark: '交通堵塞' },
  { id: 3, name: '王五', date: '2023-09-01', status: '缺勤', time: '', remark: '请假' }
])

const searchForm = ref({
  name: '',
  date: null as any
})

const currentPage = ref(1)
const pageSize = ref(10)
const total = ref(3)

const getTagType = (status: string) => {
  switch (status) {
    case '正常': return 'success'
    case '迟到': return 'warning'
    case '缺勤': return 'danger'
    case '早退': return 'info'
    default: return ''
  }
}

const handleSearch = () => {
  // 模拟搜索
  console.log('搜索条件:', searchForm.value)
}

const handleAdd = () => {
  // 模拟新增
  console.log('新增考勤')
}

const handleSizeChange = (size: number) => {
  pageSize.value = size
}

const handleCurrentChange = (current: number) => {
  currentPage.value = current
}

onMounted(() => {
  // 模拟加载数据
  console.log('加载考勤数据')
})
</script>

<style scoped>
.attendance-container {
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
</style>
