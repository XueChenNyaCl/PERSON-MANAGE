<template>
  <div class="notice-container">
    <el-card>
      <template #header>
        <div class="card-header">
          <span>通知管理</span>
          <el-button type="primary" @click="handleAdd">发布通知</el-button>
        </div>
      </template>
      <el-form :inline="true" :model="searchForm" class="search-form">
        <el-form-item label="标题">
          <el-input v-model="searchForm.title" placeholder="请输入标题"></el-input>
        </el-form-item>
        <el-form-item label="类型">
          <el-select v-model="searchForm.type" placeholder="请选择类型">
            <el-option label="学校公告" value="school"></el-option>
            <el-option label="班级通知" value="class"></el-option>
          </el-select>
        </el-form-item>
        <el-form-item>
          <el-button type="primary" @click="handleSearch">搜索</el-button>
        </el-form-item>
      </el-form>
      <el-table :data="noticeList" style="width: 100%">
        <el-table-column prop="id" label="ID" width="80"></el-table-column>
        <el-table-column prop="title" label="标题"></el-table-column>
        <el-table-column prop="type" label="类型">
          <template #default="scope">
            {{ scope.row.type === 'school' ? '学校公告' : '班级通知' }}
          </template>
        </el-table-column>
        <el-table-column prop="author" label="发布人" width="120"></el-table-column>
        <el-table-column prop="createTime" label="发布时间" width="180"></el-table-column>
        <el-table-column label="操作" width="150">
          <template #default="scope">
            <el-button type="primary" size="small" @click="handleView(scope.row)">查看</el-button>
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
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'

interface Notice {
  id: number
  title: string
  type: 'school' | 'class'
  content: string
  author: string
  createTime: string
}

const noticeList = ref<Notice[]>([
  { id: 1, title: '开学通知', type: 'school', content: '新学期将于9月1日正式开始，请同学们做好准备。', author: '教务处', createTime: '2023-08-15 10:00:00' },
  { id: 2, title: '家长会通知', type: 'class', content: '高一(1)班将于9月10日召开家长会，请家长准时参加。', author: '班主任', createTime: '2023-09-05 14:00:00' },
  { id: 3, title: '运动会安排', type: 'school', content: '学校将于10月1日举办秋季运动会，具体安排如下。', author: '体育组', createTime: '2023-09-20 09:00:00' }
])

const searchForm = ref({
  title: '',
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
  console.log('发布通知')
}

const handleView = (row: Notice) => {
  // 模拟查看
  console.log('查看通知:', row)
}

const handleDelete = (id: number) => {
  // 模拟删除
  console.log('删除通知:', id)
}

const handleSizeChange = (size: number) => {
  pageSize.value = size
}

const handleCurrentChange = (current: number) => {
  currentPage.value = current
}

onMounted(() => {
  // 模拟加载数据
  console.log('加载通知数据')
})
</script>

<style scoped>
.notice-container {
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
