import './style.css';
import router from './router';
import { renderLoginPage } from './pages/login-page';
import { renderStatusPage } from './pages/status-page';
import { renderStartPage } from './pages/start-page';

import { attachConsole } from '@tauri-apps/plugin-log';
await attachConsole(); // show tauri backend logs in console

router
  .on('/', renderStartPage)
  .on('/login', renderLoginPage)
  .on('/status', renderStatusPage)
  .on('*', () => {
    router.navigate('/');
  })
  .resolve();
