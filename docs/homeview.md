# HomeView 实现详情

## 样式实现
- 完全按照 `x:\works\code\ccccc\model\樱花` 目录下的设计实现
- 使用了樱花设计的 CSS 样式，包括响应式布局
- 实现了滚动动画效果，如计划卡片的渐入动画
- 实现了关于页面的星星动画效果
- 使用了 iconfont 图标字体

## 图片使用
- 图片路径：`/src/assets/images/`
- 图片命名格式：`school_manage_01.jpg`, `school_manage_02.jpg`, `school_manage_03.jpg`

### 图片序号及用途
1. **school_manage_01.jpg**：介绍部分背景图片
2. **school_manage_02.jpg**：首页背景图片和介绍部分右侧图片
3. **school_manage_03.jpg**：定价部分背景图片

### 原图片对应关系
- **school_manage_01.jpg**：对应原图片 `x:\works\code\ccccc\image\6ed11b835ebea2b4ef5f5c1ee30ee053.jpg`
- **school_manage_02.jpg**：对应原图片 `x:\works\code\ccccc\image\92682d71dd3affc3c564bed54954c8c9.jpg`
- **school_manage_03.jpg**：对应原图片 `x:\works\code\ccccc\image\e76f681342a2e78e87cc15344884aed8.jpg`

## 页面结构
1. **首页横幅**：包含标题、标语和操作按钮
2. **介绍部分**：包含系统功能介绍和右侧图片
3. **定价部分**：包含基础版、专业版和企业版三个套餐
4. **关于部分**：包含系统简介和联系按钮

## 功能实现
- 导航栏滚动效果
- 平滑滚动到各个部分
- 计划卡片的渐入动画
- 关于页面的星星动画
- 响应式布局，适配不同屏幕尺寸

## 技术栈
- Vue 3 + TypeScript
- Vite
- CSS3 动画
- JavaScript 滚动事件处理