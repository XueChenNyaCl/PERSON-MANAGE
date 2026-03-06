<template>
  <Teleport to=".content">
    <Transition name="page-transition">
      <div v-if="isVisible" class="page-transition-overlay">
        <div class="stars-line-container">
          <span
            v-for="(_star, index) in lineStars"
            :key="index"
            class="line-star"
            :style="{ '--delay': `${index * 0.15}s`, '--start-x': `${startPositions[index]}px` }"
          >
            ✦
          </span>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'

const props = defineProps<{
  duration?: number
}>()

const isVisible = ref(false)
const router = useRouter()
const animationDuration = props.duration || 800

const lineStars = ref([{}, {}, {}])
const startPositions = [-200, 0, 200]

let isTransitioning = false

const showTransition = () => {
  if (isTransitioning) return
  isTransitioning = true
  isVisible.value = true
  
  setTimeout(() => {
    isVisible.value = false
    isTransitioning = false
  }, animationDuration)
}

onMounted(() => {
  router.afterEach(() => {
    showTransition()
  })
})

defineExpose({
  show: showTransition
})
</script>

<style scoped>
.page-transition-overlay {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  z-index: 9999;
  pointer-events: none;
  display: flex;
  justify-content: center;
  align-items: center;
  background: rgba(253, 248, 240, 0.95);
}

.stars-line-container {
  display: flex;
  gap: 20px;
  align-items: center;
  justify-content: center;
}

.line-star {
  font-size: 48px;
  color: #8B4513;
  opacity: 0;
  transform: translateX(var(--start-x)) scale(0);
  animation: starConnect 0.4s ease forwards;
  animation-delay: var(--delay);
  text-shadow: 0 0 15px rgba(139, 69, 19, 0.6);
}

@keyframes starConnect {
  0% {
    opacity: 0;
    transform: translateX(var(--start-x)) scale(0) rotate(-180deg);
  }
  60% {
    opacity: 1;
    transform: translateX(0) scale(1.3) rotate(0deg);
  }
  100% {
    opacity: 1;
    transform: translateX(0) scale(1) rotate(0deg);
  }
}

.page-transition-enter-active {
  animation: fadeIn 0.2s ease;
}

.page-transition-leave-active {
  animation: fadeOut 0.4s ease;
}

@keyframes fadeIn {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

@keyframes fadeOut {
  from {
    opacity: 1;
  }
  to {
    opacity: 0;
  }
}
</style>
