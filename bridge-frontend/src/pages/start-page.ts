import { invoke } from '@tauri-apps/api/core';
import router from '../router';

export function renderStartPage() {
  document.querySelector<HTMLDivElement>('#app')!.innerHTML = `
    <div>
      <h1>SP2Any Bridge</h1>
      <div>Trying to login...</div>
    </div>
  `;

  invoke('login_with_stored_credentials')
    .then(token => {
      localStorage.setItem('jwt', token as string);
      router.navigate('/status');
    })
    .catch(error => {
      console.error('Failed to login with stored credentials:', error);
      router.navigate('/login');
    });
}