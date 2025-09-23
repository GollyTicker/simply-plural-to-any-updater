<template>
  <div class="config-container">
    <h1>Config</h1>
    <form @submit.prevent="saveConfigAndRestart" autocomplete="off">
      <div class="config-grid">
        <div class="config-item">
          <label for="wait_seconds">Wait Seconds</label>
          <input id="wait_seconds" type="number" v-model.number="config.wait_seconds" />
        </div>
        <div class="config-item">
          <label for="system_name">System Name</label>
          <input id="system_name" type="text" v-model="config.system_name" />
        </div>
        <div class="config-item">
          <label for="status_prefix">Status Prefix</label>
          <input id="status_prefix" type="text" v-model="config.status_prefix" />
        </div>
        <div class="config-item">
          <label for="status_no_fronts">Status No Fronts</label>
          <input id="status_no_fronts" type="text" v-model="config.status_no_fronts" />
        </div>
        <div class="config-item">
          <label for="status_truncate_names_to">Status Truncate Names To</label>
          <input id="status_truncate_names_to" type="number" v-model.number="config.status_truncate_names_to" />
        </div>
        <div class="config-item">
          <label for="enable_discord">Enable Discord</label>
          <input id="enable_discord" type="checkbox" v-model="config.enable_discord" />
        </div>
        <div class="config-item">
          <label for="enable_discord_status_message">Enable Discord Status Message</label>
          <input id="enable_discord_status_message" type="checkbox" v-model="config.enable_discord_status_message" />
        </div>
        <div class="config-item">
          <label for="enable_vrchat">Enable VRChat</label>
          <input id="enable_vrchat" type="checkbox" v-model="config.enable_vrchat" />
        </div>
        <div class="config-item">
          <label for="simply_plural_token">Simply Plural Token</label>
          <input id="simply_plural_token" type="password" :value="config.simply_plural_token?.secret"
            @input="setSecret('simply_plural_token', $event)" />
        </div>
        <div class="config-item">
          <label for="discord_status_message_token">Discord Status Message Token</label>
          <input id="discord_status_message_token" type="password" :value="config.discord_status_message_token?.secret"
            @input="setSecret('discord_status_message_token', $event)" />
        </div>
        <div class="config-item">
          <label for="vrchat_username">VRChat Username</label>
          <input id="vrchat_username" type="password" :value="config.vrchat_username?.secret"
            @input="setSecret('vrchat_username', $event)" />
        </div>
        <div class="config-item">
          <label for="vrchat_password">VRChat Password</label>
          <input id="vrchat_password" type="password" :value="config.vrchat_password?.secret"
            @input="setSecret('vrchat_password', $event)" />
        </div>
      </div>
      <button type="submit">Save and Restart</button>
      <p id="config-update-status">{{ status }}</p>
    </form>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, type Ref } from 'vue';
import type { Decrypted, UserConfigDbEntries } from '@/sp2any.bindings';
import { sp2any_api } from '@/sp2any_api';

const config: Ref<UserConfigDbEntries> = ref({});
type SecretKeys = "simply_plural_token" | "vrchat_password" | "vrchat_cookie" | "vrchat_username" | "discord_status_message_token";

const status = ref('');

function setSecret(key: SecretKeys, event: Event) {
  const target = event.target as HTMLInputElement;
  if (target.value !== "") {
    config.value[key] = <Decrypted>{ secret: target.value };
  }
  else {
    config.value[key] = undefined;
  }
}

async function fetchConfig() {
  try {
    config.value = await sp2any_api.get_config();
    console.log("Received user config: ", config.value);
  } catch (e) {
    console.warn(e);
  }
};

async function saveConfigAndRestart() {
  try {
    await sp2any_api.set_config_and_restart(config.value);
    status.value = 'Config saved successfully and restarted updaters!';
  } catch (e) {
    console.warn(e);
    status.value = 'Failed to save config and restart updaters.';
  }
};

onMounted(async () => {
  await fetchConfig();
});
</script>

<style scoped>
.config-container {
  padding: 2rem;
  font-family: sans-serif;
}

.config-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 1.5rem;
  margin-top: 1.5rem;
}

.config-item {
  display: flex;
  flex-direction: column;
}

.config-item label {
  font-weight: bold;
  margin-bottom: 0.5rem;
}

.config-item input {
  padding: 0.5rem;
  border: 1px solid #ccc;
  border-radius: 4px;
}

button {
  margin-top: 1.5rem;
  padding: 0.75rem 1.5rem;
  border: none;
  border-radius: 4px;
  background-color: #007bff;
  color: white;
  font-weight: bold;
  cursor: pointer;
}

button:hover {
  background-color: #0056b3;
}
</style>
