
import { createRouter, createWebHistory } from 'vue-router';
import Start from './components/Start.vue';
import Status from './components/Status.vue';
import Config from './components/Config.vue';
import DiscordAuthButton from './components/DiscordAuthButton.vue';

const routes: any = [
  { path: '/', component: Start },
  { path: '/status', component: Status },
  { path: '/config', component: Config },
  { path: '/discord-oauth', compoment: DiscordAuthButton }
];

const router = createRouter({
  history: createWebHistory(),
  routes,
});

export default router;
