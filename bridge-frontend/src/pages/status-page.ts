import { invoke } from '@tauri-apps/api/core';
import router from '../router';

export function renderStatusPage() {
  document.querySelector<HTMLDivElement>('#app')!.innerHTML = `
    <div>
      <h1>Bridge Connected</h1>
      <button id="logout-button">Disconnect and Logout</button>
    </div>
  `;

  const logoutButton = document.querySelector<HTMLButtonElement>('#logout-button');
  logoutButton?.addEventListener('click', async () => {
    localStorage.removeItem('jwt');
    try {
      await invoke('clear_credentials');
    } catch (e) {
      console.error("Failed to clear credentials on backend", e)
    }
    router.navigate('/');
  });
}