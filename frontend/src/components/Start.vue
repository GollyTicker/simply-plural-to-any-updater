<template>
  <div>
    <h1>Welcome</h1>
  </div>
</template>

<script setup lang="ts">
import router from '@/router';
import { onMounted } from 'vue';
defineProps<{
  
}>()

onMounted(() => {
  console.log("sdhsdf")
  router.push('/login');
})
</script>
