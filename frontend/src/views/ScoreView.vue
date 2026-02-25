<template>
  <div class="score-container">
    <el-card>
      <template #header>
        <div class="card-header">
          <span>评分管理</span>
          <el-button type="primary" @click="handleAdd">新增评分</el-button>
        </div>
      </template>
      <el-form :inline="true" :model="searchForm" class="search-form">
        <el-form-item label="姓名">
          <el-input v-model="searchForm.name" placeholder="请输入姓名"></el-input>
        </el-form-item>
        <el-form-item label="类型">
          <el-select v-model="searchForm.type" placeholder="请选择类型">
            <el-option label="个人分" value="person"></el-option>
            <el-option label="小组分" value="group"></el-option>
          </el-select>
        </el-form-item>
        <el-form-item>
          <el-button type="primary" @click="handleSearch">搜索</el-button>
        </el-form-item>
      </el-form>
      <el-table :data="scoreList" style="width: 100%">
        <el-table-column prop="id" label="ID" width="80"></el-table-column>
        <el-table-column prop="name" label="姓名/小组名"></el-table-column>
        <el-table-column prop="type" label="类型">
          <template #default="scope">
            {{ scope.row.type === 'person' ? '个人分' : '小组分' }}
          </template>
        </el-table-column>
        <el-table-column prop="score" label="分数" width="100">
          <template #default="scope">
            <span :class="scope.row.score > 0 ? 'score-positive' : 'score-negative'">
              {{ scope.row.score > 0 ? '+' : '' }}{{ scope.row.score }}
            </span>
          </template>
        </el-table-column>
        <el-table-column prop="reason" label="原因"></el-table-column>
        <el-table-column prop="createTime" label="创建时间" width="180"></el-table-column>
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

interface Score {
  id: number
  name: string
  type: 'person' | 'group'
  score: number
  reason: string
  createTime: string
}

const scoreList = ref<Score[]>([
  { id: 1, name: '张三', type: 'person', score: 10, reason: '作业完成优秀', createTime: '2023-09-01 10:00:00' },
  { id: 2, name: '李四', type: 'person', score: -5, reason: '迟到', createTime: '2023-09-01 08:15:00' },
  { id: 3, name: '第一小组', type: 'group', score: 20, reason: '团队合作优秀', createTime: '2023-09-01 15:00:00' }
])

const searchForm = ref({
  name: '',
  type: ''
})

const currentPage = ref(1)
const pageSize = ref(10)
const total = ref(3)

const handleSearch = () => {
  // 模拟搜索
  console.log('搜索条件:', searchForm.value)
}

const handleAdd = () => {
  // 模拟新增
  console.log('新增评分')
}

const handleSizeChange = (size: number) => {
  pageSize.value = size
}

const handleCurrentChange = (current: number) => {
  currentPage.value = current
}

onMounted(() => {
  // 模拟加载数据
  console.log('加载评分数据')
})
</script>

<style scoped>
.score-container {
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

.score-positive {
  color: #67c23a;
  font-weight: bold;
}

.score-negative {
  color: #f56c6c;
  font-weight: bold;
}
</style>
