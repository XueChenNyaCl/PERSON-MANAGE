<template>
  <router-view />
  <PageTransition :duration="800" />
</template>

<script setup lang="ts">
import PageTransition from './components/PageTransition.vue'
</script>

<style src="@/styles/main.css"></style>
