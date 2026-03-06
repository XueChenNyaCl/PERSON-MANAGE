<template>
  <Transition name="page-transition" @after-leave="onTransitionEnd">
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
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import '@styles/page-transition.css'

const props = defineProps<{
  duration?: number
}>()

const emit = defineEmits<{
  (e: 'animation-end'): void
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
  }, animationDuration)
}

const onTransitionEnd = () => {
  isTransitioning = false
  emit('animation-end')
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
