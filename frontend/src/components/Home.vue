<template>
  <div>
    <h1>Updaters Status</h1>
    <ul>
      <li v-for="upt in updaters" :key="upt.updater">
        {{ upt.updater }}: {{ upt.status }}
      </li>
    </ul>
  </div>
</template>

<script setup lang="ts">

import { ref, onMounted, onUnmounted, type Ref } from 'vue';
function invoke(_1: string, _2: any = undefined) {
  return "todo. fetch";
}

// todo. keep in sync with backend
type UpdaterState = {updater: string, status: any};

const updaters: Ref<UpdaterState[]> = ref([]);

let refreshViewIntervalTimer: any = null;

const fetchUpdatersState = async () => {
  try {
    updaters.value = <any>await invoke('get_updaters_state');
    console.log("get_updaters_state: ", updaters.value);
  } catch (e) {
    console.error(e);
  }
};

onMounted(async () => {
  await fetchUpdatersState();
  refreshViewIntervalTimer = setInterval(fetchUpdatersState, 3000);
});

onUnmounted(() => {
  refreshViewIntervalTimer ?? clearInterval(refreshViewIntervalTimer);
});
</script>
