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
        <a href="#price" class="title-bar other">
          <span>&#xe60e; <span>定价</span></span>
        </a>
        <a href="#about" class="title-bar other">
          <span>&#xe7e7; <span>关于</span></span>
        </a>
      </div>
      <div class="left-text">
        <h2 class="bigtext">SCHOOL MANAGE</h2>
        <h3 class="middletext">学校综合管理系统</h3>
        <div class="actions">
          <a href="/login" class="btn">登录系统</a>
          <a href="/login" class="btn">管理面板</a>
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
      <a class="float-down" href="#price">↓</a>
    </section>
    <section class="board board-price" style="--bg: url('/src/assets/images/school_manage_03.jpg')">
      <a @click="scrollToSection('price')" href="#price" id="price" class="title-bar">
        <div class="bg"></div>
        <span>&#xe60e; <span>定价</span></span>
      </a>
      <div class="bg"></div>
      <div class="plans">
        <div class="plan">
          <div class="level">
            <div class="level-icon">&#xe7ae;</div>
            <span>基础版</span>
          </div>
          <div class="quota">
            <div class="quota-item"><span>用户数量</span> <span>50 人</span></div>
            <div class="quota-item"><span>功能模块</span> <span>基础模块</span></div>
            <div class="quota-item"><span>数据存储</span> <span>10 GiB</span></div>
            <div class="quota-item"><span></span> <span>* 适合小型学校</span></div>
          </div>
          <a href="/login" class="price">免费</a>
        </div>
        <div class="plan">
          <div class="level">
            <div class="level-icon">&#xe6ba;</div>
            <span>专业版</span>
          </div>
          <div class="quota">
            <div class="quota-item"><span>用户数量</span> <span>500 人</span></div>
            <div class="quota-item"><span>功能模块</span> <span>全部模块</span></div>
            <div class="quota-item"><span>数据存储</span> <span>100 GiB</span></div>
            <div class="quota-item"><span>技术支持</span> <span>7×24 小时</span></div>
          </div>
          <a href="/login" class="price">￥200 / 月</a>
        </div>
        <div class="plan">
          <div class="level">
            <div class="level-icon">&#xe6c6;</div>
            <span>企业版</span>
          </div>
          <div class="quota">
            <div class="quota-item"><span>用户数量</span> <span>不限</span></div>
            <div class="quota-item"><span>功能模块</span> <span>定制开发</span></div>
            <div class="quota-item"><span>数据存储</span> <span>不限</span></div>
            <div class="quota-item"><span>技术支持</span> <span>专属客服</span></div>
          </div>
          <a href="/login" class="price">联系我们</a>
        </div>
      </div>
      <a class="float-down" href="#about">↓</a>
    </section>
    <section class="board board-about">
      <a href="#about" id="about" class="title-bar">
        <div class="bg" id="about-bg-s"></div>
        <span>&#xe7e7; <span>关于</span></span>
      </a>

      <div class="bg" id="about-bg"></div>
      <div class="left-text">
        <div class="middletext">学校综合管理系统</div>
        <div>
          为中小学提供日常管理解决方案，包括人事档案、考勤消费、分组评分、评选通知等功能。<br/><br/>
          系统采用插件化架构，以适应不同学校的软硬件环境，确保低学习成本，让教师、学生、家长都能快速上手。
        </div>
        <br/>
        <br/>

        <div style="margin-top: 40px" class="actions">
          <a href="/login" class="btn">联系我们</a>
          <a href="/login" class="btn">举报滥用</a>
        </div>
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

onMounted(() => {
  window.addEventListener('scroll', function (e) {
    if (document.documentElement.scrollTop > window.innerHeight * 1.5 && window.planEvent != true) {
      let waitTime = 0;
      document.querySelectorAll('.plans .plan').forEach((el) => {
        setTimeout(() => {
          el.classList.add('animated-up');
        }, waitTime);
        waitTime += 100;
      });

      document.querySelector('.board-price .plans')?.scrollLeft(270);
      window.planEvent = true;
    }
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

  window.addEventListener("scroll", function (e) {
    if (document.documentElement.scrollTop > window.innerHeight * 1.5) {
      if (window.oked != true) {
        var PI2 = Math.PI * 2;
        Math.dist = function (c, d) {
          var e = c.x - d.x;
          var f = d.y - d.y;
          return Math.sqrt(Math.pow(e, 2), Math.pow(f, 2));
        };
        var Stars = function (b) {
          if (b === undefined) {
            b = {};
          }
          var f = new Date().getTime();
          this.stars = b.stars;
          this.velocity = b.velocity || 1;
          this.radius = b.radius || 3;
          this.target = b.target;
          this.trail = false;
          this.alpha = 0;
          this.starsCounter = 1000;
          var c = { x: window.innerWidth / 2, y: window.innerHeight / 2 };
          var d = Math.sqrt(Math.pow(c.x, 2), Math.pow(c.y, 2));
          var velocity = this.velocity;
          var radius = this.radius;
          this.init = function () {
            this.canvas = document.createElement("canvas");
            const targetElement = document.querySelector(this.target);
            if (targetElement) {
              targetElement.appendChild(this.canvas);
            }
            this.context = this.canvas.getContext("2d");
            this.start();
            this.resize();
            window.addEventListener("resize", this.resize.bind(this));
          };
          this.start = function () {
            if (this.context) {
              this.context.clearRect(0, 0, this.canvas.width, this.canvas.height);
            }
            if (this.stars.length == 0) {
              for (var g = 0; g < this.starsCounter; g++) {
                this.stars.push(new e());
              }
            }
          };
          this.resize = function () {
            if (this.canvas) {
              this.canvas.width = window.innerWidth;
              this.canvas.height = window.innerHeight;
            }
            c.x = window.innerWidth / 2;
            c.y = window.innerHeight / 2;
          };
          this.animate = function () {
            window.requestAnimationFrame(this.animate.bind(this));
            this.render();
          };
          this.render = function () {
            f = new Date().getTime() * 0.00015;
            if (this.context) {
              if (this.trail) {
                this.context.fillStyle = "rgba(34, 34, 34, " + this.alpha + ")";
                this.context.fillRect(0, 0, this.canvas.width, this.canvas.height);
              } else {
                this.context.fillStyle = "rgb(34,34,34)";
                this.context.fillRect(0, 0, this.canvas.width, this.canvas.height);
              }
              for (var g = 0; g < this.stars.length; g++) {
                this.stars[g].draw(this.context);
              }
            }
          };
          var e = function () {
            this.offset = Math.random() * d;
            this.velocity = Math.random() * velocity;
            this.coords = { x: c.x + this.offset, y: c.y + this.offset };
            var g = Math.dist(this.coords, c);
            this.alpha = g / d;
            this.radius = Math.random() * radius;
            this.fillColor = "rgba(255," + ~~(Math.random() * 160) + "," + ~~(Math.random() * 255) + "," + this.alpha + ")";
            this.draw = function (h) {
              var i = f * this.velocity;
              this.coords = { x: c.x + Math.sin(i) * this.offset, y: c.y + Math.cos(i) * this.offset };
              h.fillStyle = this.fillColor;
              h.beginPath();
              h.arc(this.coords.x, this.coords.y, this.radius, 0, PI2);
              h.fill();
              h.closePath();
            };
            return this;
          };
          this.init();
          this.animate();
          return this;
        };
        var _stars = new Stars({ target: "#about-bg", stars: [] });
        var _stars2 = new Stars({ target: "#about-bg-s", stars: _stars.stars });
        window.oked = true;
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

.board:nth-child(3) .title-bar {
  width: calc(100% - 250px);
  margin-left: 250px
}

.board:nth-child(3) .title-bar .bg {
  width: calc(100% + 250px);
  margin-left: -250px
}

.board:nth-child(4) .title-bar {
  width: calc(100% - 375px);
  margin-left: 375px
}

.board:nth-child(4) .title-bar .bg {
  width: calc(100% + 375px);
  margin-left: -375px
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

.board-price {
  display: flex;
  justify-content: center;
  align-items: center
}

.board-price .plans {
  display: flex;
  align-items: stretch;
  text-align: center
}

.board-price .plans .plan {
  width: 250px;
  margin-right: 5vw;
  display: flex;
  background-color: rgba(25,25,25,.9);
  flex-direction: column;
  align-items: stretch;
  padding: 20px;
  border-radius: 20px;
  color: #cfcfcf
}

.board-price .plans .plan .level {
  margin-bottom: 25px;
  justify-content: center;
  align-items: center;
  display: flex;
  transition: all .3s;
  flex-direction: column;
  font-size: 23px
}

.board-price .plans .plan .level .level-icon {
  font-family: iconfont;
  background: linear-gradient(to bottom,rgba(25,25,25,.9) 50%,transparent 50%);
  border-radius: 50%;
  width: 80px;
  user-select: none;
  transition: all .3s;
  margin-top: -60px;
  height: 80px;
  justify-content: center;
  color: #cfcfcf;
  align-items: center;
  display: flex;
  font-size: 35px;
  margin-bottom: 10px
}

.board-price .plans .plan .quota {
  margin-bottom: 15px
}

.board-price .plans .plan .quota .quota-item {
  font-size: 15px;
  display: flex;
  padding: 5px 0;
  justify-content: space-between
}

.board-price .plans .plan:nth-last-child(1) {
  margin-right: 0
}

.board-price .plans .plan:hover {
  color: #fff;
  transform: translate(0,-3px)
}

.board-price .plans .plan:hover .level-icon {
  color: #fff
}

.board-price .plans .price {
  margin-top: auto;
  padding: 20px 10px;
  color: #cfcfcf;
  font-size: 16px;
  transition: all .3s;
  text-decoration: none;
  border-radius: 20px
}

.board-price .plans .price:hover {
  color: #fff;
  background-color: rgba(17,17,17,.5);
  transform: translate(0,-3px)
}

.board-price .plans .price:active {
  transform: scale(.98)
}

.board-about .bg {
  height: 95vh;
  background-color: #222
}

.board-about .left-text a {
  text-decoration: none
}

.board-about #about-bg canvas {
  margin-top: -5vh
}

.board-about #mobile-br {
  display: none
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

  .board:nth-child(3) .title-bar {
    width: 50vw!important;
    right: 0!important
  }

  .board:nth-child(3) .title-bar .bg {
    width: 100vw;
    margin-left: -50vw
  }

  .board:nth-child(4) .title-bar {
    width: 25vw!important;
    right: 0!important
  }

  .board:nth-child(4) .title-bar .bg {
    width: 100vw;
    margin-left: -75vw
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

  .board-price .plans {
    overflow: scroll;
    scroll-snap-type: x mandatory;
    padding: 50px 5vw;
    -webkit-overflow-scrolling: touch
  }

  .board-price .plans .plan {
    flex-shrink: 0;
    scroll-snap-align: center
  }

  .board-price .plans::after {
    content: "";
    width: 5vw;
    flex-shrink: 0
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