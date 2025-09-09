
import { createRouter, createWebHistory } from 'vue-router';
import Home from './components/Home.vue';
import Config from './components/Config.vue';
import DiscordAuthButton from './components/DiscordAuthButton.vue';

const routes: any = [
  { path: '/', component: Home },
  { path: '/config', component: Config },
  { path: '/discord-oauth', compoment: DiscordAuthButton }
];

const router = createRouter({
  history: createWebHistory(),
  routes,
});

export default router;
