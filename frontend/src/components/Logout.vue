<template>
  <div>
    <h1>Logging out...</h1>
  </div>
</template>

<script setup lang="ts">
import router from '@/router'
import { onMounted } from 'vue'

onMounted(() => {
  localStorage.removeItem('jwt')
  router.push('/')
})
</script>
