import router from '@/router'
import { ref } from 'vue'
import type { JwtString } from './sp2any.bindings'

export const loggedIn = ref<boolean>(false)

export async function getJwt(): Promise<JwtString> {
  const stored = localStorage.getItem('jwt')
  if (!stored) {
    logoutAndBackToStart()
    return Promise.reject('Not logged in.')
  }
  const result = JSON.parse(stored!)
  loggedIn.value = true
  return Promise.resolve(result)
}

export function setJwt(jwtString: JwtString) {
  localStorage.setItem('jwt', JSON.stringify(jwtString))
  loggedIn.value = true
}

export function clearJwt() {
  localStorage.removeItem('jwt')
  loggedIn.value = false
  console.log('loggedin value: ', loggedIn.value)
}

export function logoutAndBackToStart() {
  clearJwt()
  router.push('/')
}
