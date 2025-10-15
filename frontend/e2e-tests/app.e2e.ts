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
    await expect($('.config-container h1')).toHaveText('Settings');
}

async function navigateToConfig() {
    await $('a[href="/config"]').click();
}

async function configUpdateAndRestartSucceeded() {
    await expect($('#config-update-status')).toHaveText('Config saved successfully and restarted updaters!');
}

// async function configUpdateFailed() {
//     await expect($('#config-update-status')).toHaveText('Failed to save config and restart updaters.');
// }

async function register(email: string) {
    await $('#email').setValue(email);
    await $('#password').setValue('a-secure-password');
    await $('button.register-button').click();
}

async function registrationSucceeded() {
    await expect($('.status-message')).toHaveText('Registration successful! You can now log in.');
}

async function registrationFailed() {
    await expect($('.status-message')).toHaveText('Registration failed: AxiosError: Request failed with status code 500. error returned from database: duplicate key value violates unique constraint "users_email_key"');
}


describe('sp2any registration logic', () => {
    const test_email = `test-${Date.now()}@example.com`;

    it('should allow a new user to register', async () => {
        await browser.url(env.SP2ANY_BASE_URL!);
        await register(test_email);
        await registrationSucceeded();
    });

    it('should allow the new user to log in', async () => {
        // The form is already filled from the registration step
        await $('button[type="submit"]').click()
        await loggedInAndOnStatusPage();
    });

    it('should allow the user to log out', async () => {
        await navigateToLogout();
        await notLoggedIn();
    });

    it('should not allow registering with an existing email', async () => {
        await browser.url(env.SP2ANY_BASE_URL!);
        // This is the email of the default user, which already exists
        await register(TEST_EMAIL);
        await registrationFailed();
    });
});


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

        await expect($('#VRChat-status')).toHaveText('Disabled');
        await expect($('#ToPluralKit-status')).toHaveText('Running');
        await expect($('#Discord-status')).toHaveText('Starting');
    });

    it('should show the correct config values', async () => {
        await navigateToConfig();
        await loggedInAndOnConfigPage();

        await expect($('#enable_website')).toBeSelected();
        await expect($('#enable_vrchat')).not.toBeSelected();
        await expect($('#enable_discord')).toBeSelected();
        await expect($('#enable_to_pluralkit')).toBeSelected();
        await expect($('#enable_discord_status_message')).toBeSelected();

        await expect($('#wait_seconds')).toHaveValue(process.env.SECONDS_BETWEEN_UPDATES!);
        await expect($('#website_system_name')).toHaveValue(process.env.WEBSITE_SYSTEM_NAME!);
        await expect($('#website_url_name')).toHaveValue(process.env.WEBSITE_URL_NAME!);

        await expect($('#status_prefix')).toHaveValue("")
        await expect($('#status_no_fronts')).toHaveValue("");
        await expect($('#status_truncate_names_to')).toHaveValue("");

        await expect($('#simply_plural_token')).toHaveValue(process.env.SPS_API_TOKEN!);
        await expect($('#discord_status_message_token')).toHaveValue(process.env.DISCORD_STATUS_MESSAGE_TOKEN!);
    });

    it('should be able to disable discord and pluralkit', async () => {
        await $('#enable_to_pluralkit').click();
        await $('#enable_discord').click();

        await $('button[type="submit"]').click();
        await configUpdateAndRestartSucceeded();

        await navigateToStatus();
        await loggedInAndOnStatusPage();

        await expect($('#VRChat-status')).toHaveText('Disabled');
        await expect($('#ToPluralKit-status')).toHaveText('Disabled');
        await expect($('#Discord-status')).toHaveText('Disabled');
        await expect($('#fronting-status-example')).toHaveText('F: Annalea 💖 A., Borgn B., Daenssa 📶 D., Cstm First');
    });

    it('should be able to re-enable discord and to-pluralkit', async () => {
        await navigateToConfig();
        await loggedInAndOnConfigPage();

        await $('#enable_to_pluralkit').click();
        await $('#enable_discord').click();

        await $('button[type="submit"]').click();
        await configUpdateAndRestartSucceeded();

        await navigateToStatus();
        await loggedInAndOnStatusPage();

        await expect($('#VRChat-status')).toHaveText('Disabled');
        await expect($('#ToPluralKit-status')).toHaveText('Running');
        await expect($('#Discord-status')).toHaveText('Starting');
        await expect($('#fronting-status-example')).toHaveText('F: Annalea 💖 A., Borgn B., Daenssa 📶 D., Cstm First');
    });

    // todo. fix test. when running manually in browser, the field is correctly emptied and an error happens.
    // but the test automation doesn't correctly set the field to empty :/
    // it('should reject invalid configuration', async () => {
    //     await navigateToStatus(); // reset config update status text
    //     await navigateToConfig();
    //     await loggedInAndOnConfigPage();

    //     await expect($('#enable_website')).toBeSelected();
    //     await $('#website_system_name').setValue("");

    //     await expect($('#website_system_name')).toHaveValue("");

    //     await $('button[type="submit"]').click();
    //     await configUpdateFailed();
    // });

    // todo. fix this test
    // it('should correctly save an empty string as an optional value and correctly process numbers', async () => {
    //     await navigateToConfig();
    //     await loggedInAndOnConfigPage();

    //     // Set a value and save it
    //     await $('#wait_seconds').setValue("");
    //     await expect($('#wait_seconds')).toHaveValue("");
    //     await $('button[type="submit"]').click();
    //     await configUpdateAndRestartSucceeded();

    //     // The config is re-fetched on navigation, so the value should be gone
    //     await navigateToStatus();
    //     await navigateToConfig();
    //     await expect($('#wait_seconds')).toHaveValue('');
    // });
});
