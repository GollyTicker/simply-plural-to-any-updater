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

<style scoped>
#app {
  padding-top: 4em; /* Add padding to prevent content from overlapping with the nav bar */
}

nav {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  padding: 1em;
  background-color: #f8f9fa;
  border-bottom: 2px solid #dee2e6;
  z-index: 1000;
}

nav a {
  font-weight: bold;
  color: #2c3e50;
  text-decoration: none;
  padding: 0.5em 1em;
  border: 1px solid transparent;
  border-radius: 0.25rem;
}

nav a:hover {
  color: var(--color-primary);
  background-color: var(--color-background-mute);
}
</style>