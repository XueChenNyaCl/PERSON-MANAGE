<template>
  <div class="section-background">
    <div class="circles-background">
      <svg 
        viewBox="0 0 800 800" 
        preserveAspectRatio="xMidYMax meet"
        class="circles-svg"
      >
        <defs>
          <filter id="circleShadow" x="-20%" y="-20%" width="140%" height="140%">
            <feDropShadow dx="0" dy="2" stdDeviation="3" flood-color="rgba(22, 93, 255, 0.15)" />
          </filter>
        </defs>
        
        <g :transform="`translate(${centerX}, ${centerY})`">
          <!-- 内圆 -->
          <g class="circle-group" style="transform-origin: 0 0">
            <circle
              class="circle-outer"
              cx="0"
              cy="0"
              r="360"
              fill="none"
              stroke="#165DFF"
              stroke-width="2"
              filter="url(#circleShadow)"
            />
            <!-- 内圆上的8颗星星 -->
            <text
              v-for="i in 8"
              :key="`outer-${i}`"
              class="star-on-circle star-outer"
              :x="getStarX(360, i, 8)"
              :y="getStarY(360, i, 8)"
              text-anchor="middle"
              dominant-baseline="middle"
              font-size="16"
              fill="#165DFF"
            >✦</text>
          </g>
          
          <!-- 中圆 -->
          <g class="circle-group" style="transform-origin: 0 0">
            <circle
              class="circle-middle"
              cx="0"
              cy="0"
              r="480"
              fill="none"
              stroke="#4080FF"
              stroke-width="1.8"
              filter="url(#circleShadow)"
            />
            <!-- 中圆上的6颗星星 -->
            <text
              v-for="i in 6"
              :key="`middle-${i}`"
              class="star-on-circle star-middle"
              :x="getStarX(480, i, 6)"
              :y="getStarY(480, i, 6)"
              text-anchor="middle"
              dominant-baseline="middle"
              font-size="14"
              fill="#4080FF"
            >✦</text>
          </g>
          
          <!-- 外圆 -->
          <g class="circle-group" style="transform-origin: 0 0">
            <circle
              class="circle-inner"
              cx="0"
              cy="0"
              r="600"
              fill="none"
              stroke="#6BA0FF"
              stroke-width="1.6"
              filter="url(#circleShadow)"
            />
            <!-- 外圆上的4颗星星 -->
            <text
              v-for="i in 4"
              :key="`inner-${i}`"
              class="star-on-circle star-inner"
              :x="getStarX(600, i, 4)"
              :y="getStarY(600, i, 4)"
              text-anchor="middle"
              dominant-baseline="middle"
              font-size="12"
              fill="#6BA0FF"
            >✦</text>
          </g>
        </g>
      </svg>
    </div>
    <div class="section-content">
      <slot></slot>
    </div>
  </div>
</template>

<script setup lang="ts">
const centerX = 400
const centerY = 1000

const getStarX = (radius: number, index: number, total: number) => {
  const angle = (index / total) * 2 * Math.PI
  return radius * Math.cos(angle)
}

const getStarY = (radius: number, index: number, total: number) => {
  const angle = (index / total) * 2 * Math.PI
  return radius * Math.sin(angle)
}
</script>

<style scoped>
.section-background {
  position: relative;
  min-height: 100%;
}

.circles-background {
  position: fixed;
  bottom: 0;
  left: 200px;
  right: 0;
  height: 600px;
  overflow: hidden;
  pointer-events: none;
  z-index: 0;
}

.circles-svg {
  width: 100%;
  height: 100%;
  display: block;
}

.circle-group {
  transform-origin: 0 0;
}

.circle-outer {
  animation: rotateOuter 22s linear infinite;
  transform-origin: 0 0;
}

.circle-middle {
  animation: rotateMiddle 28s linear infinite reverse;
  transform-origin: 0 0;
}

.circle-inner {
  animation: rotateInner 35s linear infinite;
  transform-origin: 0 0;
}

.star-on-circle {
  pointer-events: none;
  transform-origin: 0 0;
}

.star-outer {
  animation: rotateOuter 22s linear infinite;
}

.star-middle {
  animation: rotateMiddle 28s linear infinite reverse;
}

.star-inner {
  animation: rotateInner 35s linear infinite;
}

@keyframes rotateOuter {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

@keyframes rotateMiddle {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

@keyframes rotateInner {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

.section-content {
  position: relative;
  z-index: 1;
}

@media (max-width: 768px) {
  .circles-background {
    height: 300px;
  }
}
</style>
