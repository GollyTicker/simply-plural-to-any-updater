
import { createRouter, createWebHistory } from 'vue-router';
import Start from './components/Start.vue';
import Status from './components/Status.vue';
import Config from './components/Config.vue';
import DiscordAuthButton from './components/DiscordAuthButton.vue';
import Login from './components/Login.vue';

const routes: any = [
  { path: '/', component: Start },
  { path: '/login', component: Login },
  { path: '/status', component: Status },
  { path: '/config', component: Config },
  { path: '/discord-oauth', component: DiscordAuthButton }
];

const router = createRouter({
  history: createWebHistory(),
  routes,
});

export default router;
