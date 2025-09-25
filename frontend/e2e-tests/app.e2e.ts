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

async function navigateToLogout() {
    await $('a[href="/logout"]').click();
}

async function loggedInAndOnStatusPage() {
    await expect($('#status-page-title')).toHaveText("Updaters Status")
}

async function navigateToStatus() {
    await $('a[href="/status"]').click();
}

async function loggedInAndOnConfigPage() {
    await expect($('.config-container h1')).toHaveText('Config');
}

async function navigateToConfig() {
    await $('a[href="/config"]').click();
}

async function configUpdateAndRestartSucceeded() {
    await expect($('#config-update-status')).toHaveText('Config saved successfully and restarted updaters!');
}


describe('sp2any login logic', () => {
    it('should be intially not logged in', async () => {
        await browser.url(env.SP2ANY_BASE_URL!);
        await notLoggedIn()
    })

    it('can then be logged in to see updater status', async () => {
        await login()
        await loggedInAndOnStatusPage()
    })

    it('can logout and then re-login', async () => {
        await navigateToLogout();
        await notLoggedIn();

        await login();
        await loggedInAndOnStatusPage();
    });

    it('should redirect to login on invalid jwt', async () => {
        await browser.execute(() => {
            window.localStorage.setItem('jwt', '{"inner":"invalid-jwt"}');
        });

        await navigateToConfig();
        await notLoggedIn();
    });
});

describe('sp2any updater status and config save and restarts', () => {
    it('should show the correct updater status', async () => {
        await browser.url(env.SP2ANY_BASE_URL!);
        await login()
        await loggedInAndOnStatusPage()

        await expect($('#vrchat-status')).toHaveText('Running');
        await expect($('#discord-status')).toHaveText('Starting');
    });

    it('should show the correct config values', async () => {
        await navigateToConfig();
        await loggedInAndOnConfigPage();

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

    it('should be able to disable discord and vrchat', async () => {
        await $('#enable_vrchat').click();
        await $('#enable_discord').click();

        await $('button[type="submit"]').click();
        await configUpdateAndRestartSucceeded();

        await navigateToStatus();
        await loggedInAndOnStatusPage();

        await expect($('#vrchat-status')).toHaveText('Disabled');
        await expect($('#discord-status')).toHaveText('Disabled');
    });

    it('should be able to re-enable discord and vrchat', async () => {
        await navigateToConfig();
        await loggedInAndOnConfigPage();

        await $('#enable_vrchat').click();
        await $('#enable_discord').click();

        await $('button[type="submit"]').click();
        await configUpdateAndRestartSucceeded();

        await navigateToStatus();
        await loggedInAndOnStatusPage();

        await expect($('#vrchat-status')).toHaveText('Running');
        await expect($('#discord-status')).toHaveText('Starting');
    });
});
