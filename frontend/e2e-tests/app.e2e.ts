import { expect, $, browser } from '@wdio/globals'
import '@wdio/globals'
import 'mocha'
import { env } from 'process';

const TEST_EMAIL = "test@example.com";
const TEST_PASSWORD = "m?3yp%&wdS+";

async function notLoggedIn() {
    await expect($('button[type="submit"]')).toHaveText("Login")
}

async function login(password?: string) {
    await $('#email').setValue(TEST_EMAIL)
    await $('#password').setValue(password ?? TEST_PASSWORD)

    await $('button[type="submit"]').click()
}

// async function logout() {
//     await $('#logout-button').click()
// }

async function loggedInAndConnected() {
    await expect($('#status-page-title')).toHaveText("Updaters Status")
}

describe('sp2any-bridge login flow', () => {
    it('should be intially not logged in', async () => {
        await browser.url(env.SP2ANY_BASE_URL!)
        await notLoggedIn()
    })

    it('can then be logged in to see updater status', async () => {
        await login()
        await loggedInAndConnected()
    })

    // it('can then be logged out and disconnected', async () => {
    //     await logout()
    //     await notLoggedIn()
    // })

    // it('automatically re-logins after reload if logged in before', async () => {
    //     await login()
    //     await loggedInAndConnected()
    //     await browser.reloadSession()
    //     await loggedInAndConnected()
    // })

    // it('should show an error for wrong password', async () => {
    //     await logout()
    //     await login("wrong password")

    //     await expect($('#login-status')).toHaveText("Invalid login. Please try again.")
    // })
})
