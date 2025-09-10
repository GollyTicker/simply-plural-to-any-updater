<template>
  <div id="app">
    <nav>
      <div id="app-header">
        <img src="/favicon.png" alt="logo" />
        <h1 class="title">SP2Any</h1>
      </div>
      <div>
        | <router-link to="/status">Status</router-link>
        | <router-link to="/config">Settings</router-link>
        <!-- re-include, when needed: <router-link to="/discord-oauth">Discord OAuth</router-link> -->
      </div>
    </nav>
    <router-view />
  </div>
</template>

<script setup lang="ts">
</script>

<style scoped>
#app {
  padding-top: 60px; /* Add padding to prevent content from overlapping with the nav bar */
}

nav {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  height: 60px;
  padding: 0 1em;
  background-color: #f8f9fa;
  border-bottom: 2px solid #dee2e6;
  z-index: 1000;
  display: flex;
  justify-content: flex-start;
  align-items: center;
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

#app-header {
  display: flex;
  align-items: center;
  margin-right: 2rem;
}

#app-header img {
  width: 40px;
  height: 40px;
  margin-right: 1rem;
}

#app-header .title {
  font-size: 2em;
  font-weight: 700;
  margin: 0;
}

</style>