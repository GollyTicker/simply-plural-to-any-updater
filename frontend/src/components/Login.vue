<template>
  <div>
    <h1>Login</h1>
    <form @submit.prevent="login">
      <div>
        <label for="username">Username:</label>
        <input type="text" id="username" v-model="username" />
      </div>
      <div>
        <label for="password">Password:</label>
        <input type="password" id="password" v-model="password" />
      </div>
      <button type="submit">Login</button>
    </form>
  </div>
</template>

<script setup lang="ts">
defineProps({

})
import { ref } from 'vue';
import router from '@/router';

const username = ref('');
const password = ref('');

const login = async () => {
  // Here you would typically send a request to your backend for authentication
  // For demonstration purposes, let's just simulate a successful login
  if (username.value === 'user' && password.value === 'password') {
    alert('Login successful!');
    router.push('/status');
  } else {
    alert('Invalid credentials');
  }
};
</script>
