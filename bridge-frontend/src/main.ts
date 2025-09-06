import './style.css';
import { invoke } from '@tauri-apps/api/core';

document.querySelector<HTMLDivElement>('#app')!.innerHTML = `
  <div>
    <h1>Simply Plural Bridge</h1>
    <div id="login-status">Not logged in</div>
    <form id="login-form">
      <input type="email" id="email" placeholder="Email" required />
      <input type="password" id="password" placeholder="Password" required />
      <button type="submit">Login</button>
    </form>
  </div>
`;

const loginForm = document.querySelector<HTMLFormElement>('#login-form');
const loginStatus = document.querySelector<HTMLDivElement>('#login-status');

loginForm?.addEventListener('submit', async (e) => {
  e.preventDefault();
  const email = (document.querySelector<HTMLInputElement>('#email'))?.value;
  const password = (document.querySelector<HTMLInputElement>('#password'))?.value;

  if (email && password) {
    try {
      const token: string = await invoke('login', { email, password });
      localStorage.setItem('jwt', token);
      loginStatus!.textContent = 'Logged in!';
    } catch (error) {
      loginStatus!.textContent = `Login failed: ${error}`;
    }
  }
});
