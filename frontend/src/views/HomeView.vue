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

<style scoped>
.home-container {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
}

.bg,.bg:after {
  position: absolute;
  height: 100vh
}

@font-face {
  font-family: iconfont;
  src: url(//at.alicdn.com/t/font_2353907_e2og6dtx4cc.eot);
  src: url(//at.alicdn.com/t/font_2353907_e2og6dtx4cc.eot?#iefix) format("embedded-opentype"),url(//at.alicdn.com/t/font_2353907_e2og6dtx4cc.woff2) format("woff2"),url(//at.alicdn.com/t/font_2353907_e2og6dtx4cc.woff) format("woff"),url(//at.alicdn.com/t/font_2353907_e2og6dtx4cc.ttf) format("truetype"),url(//at.alicdn.com/t/font_2353907_e2og6dtx4cc.svg#iconfont) format("svg")
}

html {
  scroll-behavior: smooth
}

body {
  margin: 0
}

h1,h2,h3 {
  font-weight: 500;
  letter-spacing: .3rem;
  text-transform: uppercase
}

a {
  color: inherit
}

.bg {
  width: 100%;
  z-index: -1;
  top: 0;
  overflow: hidden
}

.bg:after {
  content: "";
  margin-top: -5vh;
  background: linear-gradient(to right,rgba(0,0,0,.5),rgba(0,0,0,.5)),var(--bg);
  background-size: cover;
  width: 100vw
}

.board {
  width: 100%;
  position: sticky;
  top: 5vh;
  height: 95vh;
  display: flex;
  align-items: center;
  margin-top: 5vh;
  color: #fff;
  box-shadow: 0 -5px 5px rgba(0,0,0,.06)
}

.board:nth-child(1) {
  border-radius: 0;
  top: 0vh;
  margin-top: 0;
  height: 100vh
}

.board:nth-child(1) .title-bar {
  position: initial;
  width: 125px;
  margin-top: 0;
  float: left;
  white-space: nowrap
}

.board:nth-child(2) .title-bar {
  width: calc(100% - 125px);
  margin-left: 125px
}

.board:nth-child(2) .title-bar .bg {
  width: calc(100% + 125px);
  margin-left: -125px
}



.board .title-bar-btns {
  position: absolute;
  top: 0
}

.board .title-bar {
  position: absolute;
  box-shadow: -5px -5px 5px rgba(0,0,0,.06);
  margin-top: -5vh;
  top: 0;
  box-sizing: border-box;
  align-items: center;
  border-top-left-radius: 20px;
  height: 5vh;
  display: flex;
  padding: 25px 35px;
  overflow: hidden;
  font-size: 1em;
  color: #fff
}

.board .title-bar .bg {
  z-index: -1;
  top: 0;
  left: 0
}

.board .title-bar .bg:after {
  margin-top: 0!important
}

.board .other {
  color: #cfcfcf;
  transition: all .3s
}

.board .other:hover {
  color: #fff
}

.board .left-text {
  position: absolute;
  left: 15%
}

.board .left-text .bigtext {
  margin-top: 0;
  margin-bottom: 20px;
  font-size: 60px
}

.board .left-text .middletext {
  font-size: 30px;
  margin-top: 0;
  margin-bottom: 40px
}

.board .left-text .actions {
  display: flex;
  align-items: center;
  gap: 20px
}

.board .left-text .actions .btn {
  padding: 15px 40px;
  border: 2px solid #fff;
  border-radius: 30px;
  text-decoration: none;
  color: #fff;
  transition: all .3s
}

.board .left-text .actions .btn:hover {
  background-color: #fff;
  color: #333
}

.board .left-text .actions .btn:active {
  transform: scale(.98)
}

.board .right-pic img,.board-price .plans .plan {
  transition: all .5s;
  opacity: 0;
  transform: translate(0,10vh)
}

.board .right-pic {
  position: absolute;
  user-select: none;
  right: 15%
}

.board .right-pic img {
  border-radius: 2px;
  max-width: 1388px;
  width: 40vw;
  height: auto;
  box-shadow: 0 0 20px rgba(0,0,0,.5)
}

.board .float-down {
  animation: 1s ease-in-out 0s infinite alternate float;
  animation-direction: alternate-reverse;
  position: absolute;
  font-size: 40px;
  right: calc(50% - 20px);
  bottom: 5%;
  color: rgba(255,255,255,.5);
  transition: all .3s;
  text-decoration: none
}

.board .float-down:hover {
  color: #fff
}

.board-index {
  background: linear-gradient(to right,rgba(0,0,0,.5),rgba(0,0,0,.5)),url('/src/assets/images/school_manage_02.jpg');
  background-size: cover;
  background-position: center
}



.animated-up {
  transform: translate(0,0)!important;
  opacity: 1!important
}

@keyframes float {
  from {
    transform: translate(0,0)
  }

  to {
    transform: translate(0,5px)
  }
}

::-webkit-scrollbar {
  display: none
}

@media screen and (max-width: 900px) {
  body {
    font-size:14px
  }

  .board .title-bar {
    padding: 0
  }

  .board .title-bar span {
    width: 25vw;
    text-align: center
  }

  .board:nth-child(1) .title-bar-btns {
    width: 100vw
  }

  .board:nth-child(1) .title-bar {
    position: initial;
    width: 25vw!important;
    margin-top: 0;
    float: left;
    justify-content: center
  }

  .board:nth-child(2) .title-bar {
    position: absolute;
    width: 75vw!important;
    right: 0!important
  }

  .board:nth-child(2) .title-bar .bg {
    width: 100vw;
    margin-left: -25vw
  }



  .board {
    justify-content: center
  }

  .board .left-text {
    position: initial;
    left: unset;
    text-align: center
  }

  .board .left-text .actions {
    justify-content: center
  }

  .board .left-text #mobile-br {
    display: block
  }

  .float-down {
    bottom: 10%!important
  }

  .board-intro {
    flex-direction: column
  }

  .board-intro .right-pic {
    margin-top: 30px;
    position: initial!important;
    left: unset;
    text-align: center
  }

  .board-intro .right-pic img {
    max-width: 90vw;
    width: 90vw
  }



  .left-text {
    padding: 0 20px
  }
}
</style>

<style>
/* 全局样式 */
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
  font-size: 16px;
  line-height: 1.5;
  color: #333;
  background-color: #f5f5f5;
}

/* 图标字体 */
@font-face {
  font-family: 'iconfont';
  src: url('//at.alicdn.com/t/font_2353907_e2og6dtx4cc.eot');
  src: url('//at.alicdn.com/t/font_2353907_e2og6dtx4cc.eot?#iefix') format('embedded-opentype'),
  url('//at.alicdn.com/t/font_2353907_e2og6dtx4cc.woff2') format('woff2'),
  url('//at.alicdn.com/t/font_2353907_e2og6dtx4cc.woff') format('woff'),
  url('//at.alicdn.com/t/font_2353907_e2og6dtx4cc.ttf') format('truetype'),
  url('//at.alicdn.com/t/font_2353907_e2og6dtx4cc.svg#iconfont') format('svg');
}

.title-bar-btns span {
  font-family: 'iconfont';
  margin-right: 5px;
}
</style>