<template>
  <div class="status-container">
    <h1 id="status-page-title">Updaters Status</h1>
    <div class="status-list">
      <div v-for="(status, name) in updaters" :key="name" class="status-item">
        <span class="service-name">{{ name }}</span>
        <span :class="['status-badge', 'status-' + statusKind(status!).toLowerCase()]">{{ statusKind(status!) }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">

import { ref, onMounted, onUnmounted, type Ref } from 'vue';
import type { UpdaterStatus, UserUpdatersStatuses } from '@/sp2any.bindings';
import { sp2any_api } from '@/sp2any_api';

const updaters: Ref<UserUpdatersStatuses> = ref({});

let refreshViewIntervalTimer: number | undefined = undefined;

function statusKind(status: UpdaterStatus): string {
  switch (status) {
    case 'Inactive': return status;
    case 'Running': return status;
    default: return "Error";
  }
}

const fetchUpdatersState = async () => {
  try {
    updaters.value = await sp2any_api.get_updater_status();
    console.log("get_updater_status: ", updaters.value);
  } catch (e) {
    console.warn(e);
  }
};

onMounted(async () => {
  await fetchUpdatersState();
  refreshViewIntervalTimer = setInterval(fetchUpdatersState, 5000);
});

onUnmounted(() => {
  refreshViewIntervalTimer ?? clearInterval(refreshViewIntervalTimer!);
});

</script>

<style scoped>
.status-container {
  padding: 2rem;
  font-family: sans-serif;
}

.status-list {
  margin-top: 1.5rem;
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.status-item {
  display: flex;
  justify-content: flex-start;
  align-items: center;
  padding: 1rem;
  background-color: var(--color-background-soft);
  border-radius: 5px;
}

.service-name {
  font-weight: bold;
}

.status-badge {
  padding: 0.25rem 0.75rem;
  margin-left: 2em;
  border-radius: 5px;
  background-color: black;
  color: white;
  font-weight: bold;
}

.status-running {
  background-color: green;
  color: white;
}

.status-inactive {
  background-color: gray;
  color: white;
}

.status-error {
  background-color: orange;
  color: white;
}
</style>
