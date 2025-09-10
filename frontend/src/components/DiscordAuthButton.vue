<template>
  <div class="discord-auth-test">
    <h2>Test Discord OAuth</h2>
    <p>
      This is a temporary component to test the Discord OAuth flow. You need a
      valid JWT for a user to proceed.
    </p>

    <div class="form-group">
      <label for="jwt">User JWT:</label>
      <input
        type="text"
        v-model="jwt"
        id="jwt"
        name="jwt"
        size="50"
        placeholder="Enter your JWT here"
      />
    </div>

    <button @click="authorizeWithDiscord">Authorize with Discord</button>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  
}>()

import { ref, type Ref } from "vue";

const jwt: Ref<string> = ref("");

const authorizeWithDiscord = () => {
  if (!jwt.value) {
    console.warn("Please enter a JWT!");
    return;
  }

  const jwtStruct = {
    inner: jwt.value,
  };

  const jwtJSON = JSON.stringify(jwtStruct);
  
  console.log("jwt struct as JSON:", jwtJSON);

  const clientId = "1408232222682517575";
  const redirectUri =
    "http://localhost:8080/api/user/platform/discord/oauth/callback";
  const scope = "identify";

  const oauthUrl = `https://discord.com/oauth2/authorize?client_id=${clientId}&response_type=code&redirect_uri=${encodeURIComponent(
    redirectUri
  )}&scope=${scope}&state=${encodeURIComponent(jwtJSON)}`;

  window.open(oauthUrl, "_blank"); // opens in new tab.
};
</script>

<style scoped>
.discord-auth-test {
  border: 1px solid #ccc;
  padding: 1rem;
  margin: 1rem;
  border-radius: 5px;
  max-width: 600px;
}
.form-group {
  margin-bottom: 1rem;
}
label {
  display: block;
  margin-bottom: 0.5rem;
}
input {
  padding: 0.5rem;
  width: 100%;
  box-sizing: border-box;
}
button {
  padding: 0.5rem 1rem;
  cursor: pointer;
}
</style>
