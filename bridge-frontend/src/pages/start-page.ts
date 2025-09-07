import { invoke } from '@tauri-apps/api/core';
import router from '../router';
import type { JwtString } from '../types';

export function renderStartPage() {
  document.querySelector<HTMLDivElement>('#app')!.innerHTML = `
    <div>
      <h1>SP2Any Bridge</h1>
      <div>Trying to login...</div>
    </div>
  `;

  initiate_discord_rpc_loop();

  invoke<JwtString>('login_with_stored_credentials')
    .then(token => {
      localStorage.setItem('jwt', JSON.stringify(token));
      router.navigate('/status');
    })
    .catch(error => {
      console.error('Failed to login with stored credentials:', error);
      router.navigate('/login');
    });
}

async function initiate_discord_rpc_loop() {
  try {
    await invoke('initiate_discord_rpc_loop');
    console.log('Connected to Discord RPC ...');
  } catch (e) {
    console.error(e);
  }
}

