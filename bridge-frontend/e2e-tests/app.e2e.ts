import { expect, $ } from '@wdio/globals'

const TEST_EMAIL = "test@example.com";
const TEST_PASSWORD = "m?3yp%&wdS+";

describe('sp2any-bridge login flow', () => {
  it('should be intially not logged in', async () => {
    const elem = $('#login-status')
    await expect(elem).toHaveText("Not logged in")
  })

  it('can be logged in', async () => {
    await $('#email').setValue(TEST_EMAIL)
    await $('#password').setValue(TEST_PASSWORD)

    await $('button[type="submit"]').click()

    const loginStatus = $('#login-status')
    await expect(loginStatus).toHaveText("Logged in!")
  })

  it('cannot be logged in with wrong password', async () => {
    await $('#email').setValue(TEST_EMAIL)
    await $('#password').setValue(TEST_PASSWORD + ".")

    await $('button[type="submit"]').click()

    const loginStatus = $('#login-status')
    await expect(loginStatus).toHaveText("Logged in!")
  })
})
