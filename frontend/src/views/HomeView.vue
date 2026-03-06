<template>
  <div class="home-container">
    <section class="board board-index">
      <div class="title-bar-btns">
        <a href="#index" @click="scrollToTop" id="index" class="title-bar">
          <span>&#xe600; <span>首页</span></span>
        </a>
        <a href="#intro" class="title-bar other">
          <span>&#xe617; <span>介绍</span></span>
        </a>
      </div>
      <div class="left-text">
        <h2 class="bigtext">SCHOOL MANAGE</h2>
        <h3 class="middletext">学校综合管理系统</h3>
        <div class="actions">
          <a href="/login" class="btn">登录系统</a>
        </div>
      </div>
      <a class="float-down" href="#intro">↓</a>
    </section>
    <section class="board board-intro" style="--bg: url('/src/assets/images/school_manage_01.jpg')">
      <a @click="scrollToSection('intro')" id="intro" href="#intro" class="title-bar">
        <div class="bg"></div>
        <span>&#xe617; <span>介绍</span></span>
      </a>
      <div class="bg"></div>
      <div class="left-text">
        <div class="middletext">为什么你需要学校管理系统？</div>
        <div>人事档案、考勤消费、分组评分、评选通知……<br/><br/>解决学校管理难题，就用学校综合管理系统</div>
      </div>
      <div class="right-pic">
        <img src="/src/assets/images/school_manage_02.jpg" alt="系统演示" class="video" />
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue';

const scrollToTop = () => {
  window.scrollTo({ top: 0, behavior: 'smooth' });
};

const scrollToSection = (sectionId: string) => {
  const element = document.getElementById(sectionId);
  if (element) {
    element.scrollIntoView({ behavior: 'smooth' });
  }
};

// 扩展Window接口，添加videoPlayed属性
declare global {
  interface Window {
    videoPlayed?: boolean;
  }
}

onMounted(() => {
  window.addEventListener('scroll', function () {
    if (document.documentElement.scrollTop > window.innerHeight * 0.75) {
      const video = document.querySelector('.video');
      if (video) {
        video.classList.add('animated-up');
        if (window.videoPlayed != true) {
          // 视频播放逻辑
          window.videoPlayed = true;
        }
      }
    }
  });
});
</script>

<style scoped src="@/styles/home-view.css"></style>