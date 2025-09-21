import { expect, $, browser } from '@wdio/globals'
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

async function loggedInAndConnected() {
    await expect($('#status-page-title')).toHaveText("Updaters Status")
}

describe('sp2any-bridge flow', () => {
    it('should be intially not logged in', async () => {
        await browser.url(env.SP2ANY_BASE_URL!);
        await notLoggedIn()
    })

    it('can then be logged in to see updater status', async () => {
        await login()
        await loggedInAndConnected()
    })

    it('should show the correct updater status', async () => {
        await expect($('#vrchat-status')).toHaveText('Running');
        await expect($('#discord-status')).toHaveText('Starting');
    });

    it('should show the correct config values', async () => {
        await $('a[href="/config"]').click();
        await expect($('.config-container h1')).toHaveText('Config');

        await expect($('#enable_vrchat')).toBeSelected();
        await expect($('#enable_discord')).toBeSelected();
        await expect($('#enable_discord_status_message')).not.toBeSelected();

        await expect($('#wait_seconds')).toHaveValue(process.env.SECONDS_BETWEEN_UPDATES!);
        await expect($('#system_name')).toHaveValue(process.env.SYSTEM_PUBLIC_NAME!);

        await expect($('#status_prefix')).toHaveValue("")
        await expect($('#status_no_fronts')).toHaveValue("");
        await expect($('#status_truncate_names_to')).toHaveValue("");

        await expect($('#simply_plural_token')).toHaveValue(process.env.SPS_API_TOKEN!);
        await expect($('#discord_status_message_token')).toHaveValue(process.env.DISCORD_STATUS_MESSAGE_TOKEN!);
        await expect($('#vrchat_username')).toHaveValue(process.env.VRCHAT_USERNAME!);
        await expect($('#vrchat_password')).toHaveValue(process.env.VRCHAT_PASSWORD!);
    });

    // todo. add tests where we save these values and then observe the changes.
    // manually tested that this works.
});