<template>
  <div class="markdown-content" v-html="renderedContent"></div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  content: string
}>()

// 简单的Markdown渲染函数
const renderMarkdown = (markdown: string): string => {
  let html = markdown

  // 转义HTML特殊字符
  html = html
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')

  // 标题
  html = html.replace(/^###### (.*$)/gim, '<h6>$1</h6>')
  html = html.replace(/^##### (.*$)/gim, '<h5>$1</h5>')
  html = html.replace(/^#### (.*$)/gim, '<h4>$1</h4>')
  html = html.replace(/^### (.*$)/gim, '<h3>$1</h3>')
  html = html.replace(/^## (.*$)/gim, '<h2>$1</h2>')
  html = html.replace(/^# (.*$)/gim, '<h1>$1</h1>')

  // 粗体
  html = html.replace(/\*\*(.*?)\*\*/g, '<strong>$1</strong>')
  html = html.replace(/__(.*?)__/g, '<strong>$1</strong>')

  // 斜体
  html = html.replace(/\*(.*?)\*/g, '<em>$1</em>')
  html = html.replace(/_(.*?)_/g, '<em>$1</em>')

  // 代码块
  html = html.replace(/```([\s\S]*?)```/g, '<pre><code>$1</code></pre>')

  // 行内代码
  html = html.replace(/`([^`]+)`/g, '<code>$1</code>')

  // 表格
  html = html.replace(/\|(.+)\|/g, (_match, content) => {
    const cells = content.split('|').map((cell: string) => cell.trim())
    return '<td>' + cells.join('</td><td>') + '</td>'
  })

  // 列表项
  html = html.replace(/^\* (.*$)/gim, '<li>$1</li>')
  html = html.replace(/^- (.*$)/gim, '<li>$1</li>')
  html = html.replace(/^\d+\. (.*$)/gim, '<li>$1</li>')

  // 链接
  html = html.replace(/\[([^\]]+)\]\(([^)]+)\)/g, '<a href="$2" target="_blank">$1</a>')

  // 段落
  html = html.replace(/\n\n/g, '</p><p>')
  html = '<p>' + html + '</p>'

  // 清理空段落
  html = html.replace(/<p><\/p>/g, '')

  // 换行
  html = html.replace(/\n/g, '<br>')

  return html
}

const renderedContent = computed(() => {
  return renderMarkdown(props.content)
})
</script>

<style scoped>
.markdown-content {
  line-height: 1.6;
  font-size: 14px;
}

.markdown-content :deep(h1) {
  font-size: 1.5em;
  margin-bottom: 0.5em;
  color: #333;
  border-bottom: 2px solid #eee;
  padding-bottom: 0.3em;
}

.markdown-content :deep(h2) {
  font-size: 1.3em;
  margin: 0.8em 0 0.4em;
  color: #444;
  border-bottom: 1px solid #eee;
  padding-bottom: 0.2em;
}

.markdown-content :deep(h3) {
  font-size: 1.1em;
  margin: 0.6em 0 0.3em;
  color: #555;
}

.markdown-content :deep(table) {
  width: 100%;
  border-collapse: collapse;
  margin: 1em 0;
  font-size: 13px;
}

.markdown-content :deep(th),
.markdown-content :deep(td) {
  border: 1px solid #ddd;
  padding: 8px;
  text-align: left;
}

.markdown-content :deep(th) {
  background-color: #f5f5f5;
  font-weight: bold;
}

.markdown-content :deep(tr:nth-child(even)) {
  background-color: #fafafa;
}

.markdown-content :deep(ul),
.markdown-content :deep(ol) {
  margin: 0.5em 0;
  padding-left: 1.5em;
}

.markdown-content :deep(li) {
  margin: 0.2em 0;
}

.markdown-content :deep(pre) {
  background-color: #f5f5f5;
  padding: 1em;
  border-radius: 4px;
  overflow-x: auto;
  margin: 0.5em 0;
}

.markdown-content :deep(code) {
  background-color: #f0f0f0;
  padding: 0.2em 0.4em;
  border-radius: 3px;
  font-family: 'Courier New', monospace;
  font-size: 0.9em;
}

.markdown-content :deep(pre code) {
  background-color: transparent;
  padding: 0;
}

.markdown-content :deep(a) {
  color: #409eff;
  text-decoration: none;
}

.markdown-content :deep(a:hover) {
  text-decoration: underline;
}

.markdown-content :deep(p) {
  margin: 0.5em 0;
}

.markdown-content :deep(strong) {
  color: #333;
}

.markdown-content :deep(em) {
  color: #666;
  font-style: italic;
}
</style>
