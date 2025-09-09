<template>
  <div id="app">
    <nav>
      <router-link to="/status">Status</router-link> |
      <router-link to="/config">Settings</router-link>
      <!-- re-include, when needed: <router-link to="/discord-oauth">Discord OAuth</router-link> -->
    </nav>
    <router-view />
  </div>
</template>

<script setup lang="ts">
</script>