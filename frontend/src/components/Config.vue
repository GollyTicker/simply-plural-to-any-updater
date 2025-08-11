<template>
  <div>
    <h1>Configuration</h1>
    <form @submit.prevent="saveConfig">
      <div v-for="(value, key) in config" :key="key">
        <label :for="key">{{ key }}</label>
        <input :id="key" v-model="config[key]" />
      </div>
      <button type="submit">Save & Restart</button>
    </form>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  
}>()

import { ref, onMounted } from 'vue';
function invoke(_1: string, _2: any = undefined) {
  return "todo. fetch";
}

const config = ref({});

onMounted(async () => {
  try {
    config.value = await invoke('get_config');
  } catch (e) {
    console.error(e);
  }
});

const saveConfig = async () => {
  try {
    await invoke('set_config_and_restart', { config: config.value });
  } catch (e) {
    console.error(e);
  }
};
</script>
