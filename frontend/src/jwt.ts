import router from '@/router'
import { ref } from 'vue';
import type { JwtString } from './sp2any.bindings';

export const loggedIn = ref<boolean>(false);

export function getJwt(): JwtString {
    return JSON.parse(localStorage.getItem('jwt')!);
}

export function setJwt(jwtString: JwtString) {
    localStorage.setItem('jwt', JSON.stringify(jwtString))
    loggedIn.value = true;
}

export function clearJwt() {
    localStorage.removeItem('jwt')
    loggedIn.value = false;
}

export function logoutAndBackToStart() {
    clearJwt()
    router.push('/')
}
