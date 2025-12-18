import { invoke } from '@tauri-apps/api/core'
import router from '../router'
import type { JwtString } from '../pluralsync.bindings'

export function renderStartPage() {
  document.querySelector<HTMLDivElement>('#app')!.innerHTML = `
    <div>
      <h1>Login</h1>
      <div>Logging in...</div>
    </div>
  `

  invoke<JwtString>('login_with_stored_credentials')
    .then((token) => {
      localStorage.setItem('jwt', JSON.stringify(token))
      router.navigate('/status')
    })
    .catch((error) => {
      console.warn('Failed to login with stored credentials:', error)
      router.navigate('/login')
    })
}
