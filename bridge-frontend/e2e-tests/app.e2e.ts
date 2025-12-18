import { expect, $ } from '@wdio/globals'

const TEST_EMAIL = "test@example.com";
const TEST_PASSWORD = "m?3yp%&wdS+";

async function notLoggedIn() {
    await expect($('#login-status')).toHaveText("Not logged in")
}

async function login(password?: string, baseUrl?: string) {
    await $('#email').setValue(TEST_EMAIL)
    await $('#password').setValue(password ?? TEST_PASSWORD)
    if (baseUrl) {
        await $('#sp2any-base-url-input').setValue(baseUrl)
    }

    await $('button[type="submit"]').click()
}

async function logout() {
    await $('#logout-button').click()
}

async function loggedInAndConnected() {
    await expect($('#bridge-status')).toHaveText("Connected to PluralSync and receiving updates...")
}

describe('sp2any-bridge login flow', () => {
    it('should be intially not logged in', async () => {
        await notLoggedIn()
    })

    it('can then be logged in to receive updates', async () => {
        await login()
        await loggedInAndConnected()
    })

    it('can then be logged out and disconnected', async () => {
        await logout()
        await notLoggedIn()
    })

    it('automatically re-logins after reload if logged in before', async () => {
        await login()
        await loggedInAndConnected()
        await browser.reloadSession()
        await loggedInAndConnected()
    })

    it('should show an error for wrong password', async () => {
        await logout()
        await login("wrong password")

        await expect($('#login-status')).toHaveText("Invalid login. Please try again.")
    })
});

describe('variants and base-url configuration', () => {
    it('should show @local variant by default in test setup', async () => {
        await expect($('#variant-info')).toHaveText('@local')
    })

    it('should allow changing the base url and fail login', async () => {
        await login(TEST_PASSWORD, 'http://localhost:23923')
        await expect($('#login-status')).toHaveText("Login failed: error sending request for url (http://localhost:23923/api/user/login)")
    })

    it('should allow changing the base url and succeed login', async () => {
        await login(TEST_PASSWORD, process.env.SP2ANY_BASE_URL!)
        await loggedInAndConnected()
        await expect($('#variant-info')).toHaveText('@local')
        await logout()
    })
});
