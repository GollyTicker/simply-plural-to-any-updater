import { invoke } from '@tauri-apps/api/core'
import router from '../router'
import type { JwtString } from '../sp2any.bindings'
import { listen } from '@tauri-apps/api/event'

const WEBSOCKET_RETRY_INTERVAL_MILLIS = 10 * 1000

let retryTimer: NodeJS.Timeout | undefined

export function renderStatusPage() {
  document.querySelector<HTMLDivElement>('#app')!.innerHTML = `
    <div>
      <h1>Status</h1>
      <div id="bridge-status">Connecting to SP2Any Server and local Discord client ...</div>
      <button id="logout-button">Disconnect and Logout</button>
    </div>
  `

  const logoutButton = document.querySelector<HTMLButtonElement>('#logout-button')
  logoutButton?.addEventListener('click', async () => {
    localStorage.removeItem('jwt')
    try {
      await invoke('stop_and_clear_credentials')
    } catch (e) {
      console.warn('Failed to stop and clear credentials', e)
    }
    router.navigate('/')
  })

  subscribe_to_bridge_channel()

  router.addLeaveHook('/status', (done: Function) => {
    clearTimeout(retryTimer)
    done()
  })
}

function bridgeStatus(): HTMLDivElement {
  return document.querySelector<HTMLDivElement>('#bridge-status')!
}

async function subscribe_to_bridge_channel() {
  try {
    const jwt: JwtString = JSON.parse(localStorage.getItem('jwt')!)
    await invoke('subscribe_to_bridge_channel', { jwt })
    bridgeStatus().textContent = 'Connected to SP2Any and receiving updates...'
  } catch (e) {
    console.warn(e)
    restart_websocket_connection_after_retry_interval()
    bridgeStatus().textContent = `Failed to connect to SP2Any: ${e}. Retrying in ${WEBSOCKET_RETRY_INTERVAL_MILLIS / 1000} seconds...`
  }
}

function restart_websocket_connection_after_retry_interval() {
  retryTimer = setTimeout(subscribe_to_bridge_channel, WEBSOCKET_RETRY_INTERVAL_MILLIS)
}

listen<string>('notify_user_on_status', (event) => {
  bridgeStatus().textContent = event.payload
})

listen<number>(
  'restart_websocket_connection_after_retry_interval',
  restart_websocket_connection_after_retry_interval,
)
