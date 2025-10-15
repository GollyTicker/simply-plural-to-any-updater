## Steps to first public release version

* Get new test users by posting about it in more channels
* complete features from first test users
  * primarily vrchat rate limit fix
  * possibly also system sync with pluralkit
* DONE: show example fronters in status page
* announce and get second test phase users: pk server, sp server, rl plural channel, pridevr plural channel, reddit?
* use websocket subscription to simply plural and only get the fronters + system, when it actually changes
  * and also make the discord websocket thing, that an update is sent immediately once the websocket is created
  * this also resolves the bottleneck of allowing users to set a 1s update duration
* sp2any-bridge
  * auto-update
  * DONE: auto-start on system start
* do not clean stuff by default in vrchat. make that configureable
  * add todo to adapt it to work with many other characters as well (chinese, japanese, etc.)
* Remove 'VRChat Status Name' field and change it
  * INSTEAD: change this functionality to have this new name configured in SP2Any UI!
  * and tell users in the setting page, that this is configureable
* make sure, that stuff stays useable in mobile view
* deploy first proper version
* finalize README
* DONE:
  * add quick and easy feedback field
  * add link to Discord Server
  * add link to KoFi and ask for kind donations
  * add link to source code

## Feedback after first deployment of public-test

* member privacy
  * DONE custom fronts configuration
  * DONE archived members configuration / hiding
  * make lists of members / CFs / archived members collapeble and searchable to manage large systems
  * probably start with defaults for actives / archived / CFs and integration with privacy buckets from SP
    * DONE: [privacy buckets in SimplyPlual](https://docs.apparyllis.com/docs/help/features/buckets/intro). Perhaps
      we can also instead make a singleton "SP2Any" account on SP and people can add that one as a friend.
      This way, they can simply assign SP2Any to existing privacy bucket groups and chose what should be shown.
      This is an alternative to asking the users to make a new privacy bucket with the name "SP2Any" which is then read by the API.
    * DONE: privacy bucket API doesn't seem to be documented. I'll have to reverse-engineer that.
  * bidrectional sync of privacy bucket membership and "show in SP2Any" setting
    > If I search for myself, and toggle the "show as fronting" button in SP2A, it autoadds me to the privacy bucket in SP.
    > And if I add myself to the PB in SP, it toggles me as "show as fronting"
  * DONE: add SP2Any user to config explanations and to the sp2any-deployments as a global singular
  * DONE: testing of privacy features
* DONE: websocket connection restarts
* DONE: better error messages which the users can also understand and which handle most common error paths
  * also let users know, when the VRChat 429 too many requests happen during login - so that they can try again in a day.
* vrchat rate limits hinders SP2Any users to login into VRChat. possibily related to the frequent re-deployments from the same IP-addr on the day before. can we maybe avoid logging in the user at system-startup, then the vrchat cookie already exists from a previous login? what other ways can we use to bypass the rate-limits? maybe do the login in browser instead of via the backend?
* PARTIAL DONE: Add automatic sync to PluralKit
  * DONE: SimplyPlural -> PluralKit sync
  * automatic system sync?
  * set fronter start time based correctly
    * this can be better done, once the plural fetching happens on demand to avoid exessive switches

---

## Backlog

For the next steps, it probably makes sense to announce it in more discord servers and get a larger set of users.
This way we can get even more early testers so that we can then move to the app earlier or later based on the feedback.

* checkout inspirations channel in simply plural to see how users use SP and which use cases suit make sense there
* registration logs in by default as well automatically
* security: make it such that on my private instance, only handpicked users may register and use it.
* configureable order in which fronts are shown
* make it more clear, what the people need to do make the discord bridge thing work. maybe a list of steps and if they're working.
* support large systems. i.e. members search and bulk edit.
* **BIG**: APP version so that data and tokens are securely saved in the users local smartphone
  * precondition: make queries.rs into trait and create local implementations for SQLite and Postgres
  * precondition: make HTTP requests layer between frontend and rocket server such that (1) the backend exposes itself as both http endpoints and Tauri commands (via cfg macro) and (2) the front-end uses an interface to decide whether to use Tauri invoke or HTTP requests to access the local/server back-end.
  * This might get complex... Most things should work, but probably not the wwbsocket thing for discord...
  * **alternative: PWA**. see below
* password reset for users
* BUG: when discord rich presence is disabled and the bridge is started, it connects and shows up as "running" though it doesn't show any
  rich presence in discord. this might be confusing. and also, there happens some related errors in the bridge logs which should be investigated
* add status not only for updaters but also for SP itself.
* DONE: remove `0.1.0` from sp2any bridge executable
* make sure, that during production, only my own domains are allowed and not localhost or so.
* DONE: restart updaters once in a while, just to get temporary issues out of the way (e.g. vrchat someimes just doesn't work after a re-deployment)
* DONE: make website view such that it doesn't eagery fetch data from simply plural every time but instead uses the latest values from a channel
* merge cargo crates into a single workspace to improve build times
* DONE: better split sp2any crate into what is exported to bridge-src-tauri and what is not. makes for much faster compiles
* IRRELEVANT?: reduce compile times by removing vrchatapi library and using http rest requests directly
* DONE: complete migration to webapp

* add initial suggestions by ChatAI (e.g. privacy, configurability, etc.)

---

## Initial User Feedback before first Prototype
* PARTIAL DONE: sync from and to pluralkit as well (checkout pk-rpc). most SP -> PK
* DONE add a warning, that using the discord self-botting comes with a risk for both the user and the dev
  * [artcle by discord](https://support.discord.com/hc/en-us/articles/115002192352-Automated-User-Accounts-Self-Bots)
  * [self-botting](https://gist.github.com/nomsi/2684f5692cad5b0ceb52e308631859fd)
  * [reddit 1](https://old.reddit.com/r/Discord_selfbots/comments/t9o5xf/anyone_got_banned/), [reddit 2](https://old.reddit.com/r/discordapp/comments/7nl35v/regarding_the_ban_on_selfbots/)
  * perhaps use the same approach as used by the discord chat exporter? this might actually work well.
* share with refactionvr server mods before sharing in channel
* extend SP2Any to also cover tone-tags / interaction hints as an additional use case? (e.g. IWC = interact-with-care)

---

## First deployment
* DONE: test that workflows work with deployed dev-online. WORKED WELL ON WINDOWS ON FIRST TRY!!
* DONE: add note that running the exec on windows will show a signature warning. ask users to accept it.
* DONE: deploy for discord test server users
* DONE: make SP2ANY_BASE_URL configureable for frontend-dist
* DONE: deploy on private space once and share with friend
* DONE: add hint on all deployments where it warns about which variant one is on on the nav bar
* DONE: add variant picker for sp2any-bridge, such that it even knows where to connect to!
* DONE: add download link to bridge frontend in UI
* DONE: add `enable_website` config
* DONE: fix content security policy issue where images are not allowed
* DONE: ignore dark/light mode and always use light mode in frontend and bridge-frontend
* DONE: add link to Ko-Fi for donations.

---

## PWA as a Native App Alternative

Summary of the plan to create a PWA instead of native iOS/Android apps to ensure data remains on the user's device.

### Core Architecture
- **Goal**: Avoid app stores (iOS, Android) by using an installable PWA.
- **Rust Code**: Compile the core Rust logic to **WebAssembly (WASM)**.
- **Execution**: Run the WASM module inside a **Web Worker** to handle all data processing on the client-side, preventing UI blocking. This acts as a "local backend".
- **Database**: Since direct SQLite access is not possible in a browser, use **IndexedDB** for storage. To keep SQL-based logic, a WASM-compiled version of SQLite (e.g., `sql.js`, `wa-sqlite`) can be used, which persists its data to IndexedDB.

### Background Tasks & Notifications
The primary challenge is running tasks (e.g., hourly sync, reacting to WebSocket events) when the app is in the background. A PWA cannot maintain a persistent background connection.
The solution is a **server-driven push notification system**.

**Workflow:**
1.  **Server is the Listener/Scheduler**:
    - The backend server (Rocket) listens for external WebSocket events.
    - The backend server runs a cron job for scheduled tasks (e.g., once per hour).
2.  **Server Sends Push Notification**:
    - When an event occurs, the server sends a push notification to the user's device.
3.  **Service Worker Executes Task**:
    - The push notification wakes up the PWA's Service Worker.
    - The Service Worker has a short time window to execute the required task (e.g., fetch data, update IndexedDB). A few seconds is well within the limits.

### Platform-Specific Constraints
- **Requirement**: The user must grant the PWA permission to receive push notifications.
- **Android**: More flexible. Supports "silent" push notifications that can run tasks without displaying a visible alert.
- **iOS**: More restrictive. Background execution is limited to ~30 seconds. To guarantee the task runs, the push notification may need to be user-visible (e.g., display a message or update the app's badge).

---

## Minimal PWA Test Plan

A short guide to creating a minimal PWA to test core features (installability, offline, push notifications) on Android and iOS.

### 1. Create `index.html`
The basic user interface.
```html
<!DOCTYPE html>
<html>
<head>
  <title>PWA Test</title>
  <link rel="manifest" href="/manifest.json">
  <meta name="theme-color" content="#000000"/>
</head>
<body>
  <h1>PWA Push Test</h1>
  <button id="subscribeButton">Subscribe to Push</button>
  <script src="/app.js"></script>
</body>
</html>
```

### 2. Create `manifest.json`
This file makes the web app installable.
```json
{
  "short_name": "PWA Test",
  "name": "PWA Test App",
  "icons": [
    {
      "src": "/icon.png",
      "type": "image/png",
      "sizes": "192x192"
    }
  ],
  "start_url": "/",
  "display": "standalone"
}
```
*(You will need to create a simple 192x192 `icon.png` file)*

### 3. Create `sw.js` (Service Worker)
Handles offline caching and incoming push notifications.
```javascript
// On install, cache the offline page
self.addEventListener('install', (e) => {
  e.waitUntil(
    caches.open('pwa-test-cache').then(cache => {
      return cache.add('/');
    })
  );
});

// On fetch, serve from cache if offline
self.addEventListener('fetch', (e) => {
  e.respondWith(
    caches.match(e.request).then(response => {
      return response || fetch(e.request);
    })
  );
});

// On push, show a notification
self.addEventListener('push', (e) => {
  const data = e.data.json();
  self.registration.showNotification(data.title, {
    body: 'This is a push notification!',
  });
});
```

### 4. Create `app.js` (Client-Side Logic)
Registers the service worker and handles the push subscription process.
```javascript
if ('serviceWorker' in navigator) {
  navigator.serviceWorker.register('/sw.js').then(reg => {
    console.log('Service Worker Registered');
    document.getElementById('subscribeButton').addEventListener('click', () => {
      subscribeToPush(reg);
    });
  });
}

async function subscribeToPush(registration) {
  const permission = await Notification.requestPermission();
  if (permission !== 'granted') {
    throw new Error('Permission not granted for Notification');
  }

  // IMPORTANT: Replace with your backend's VAPID public key
  const vapidPublicKey = 'YOUR_VAPID_PUBLIC_KEY';
  const subscription = await registration.pushManager.subscribe({
    userVisibleOnly: true,
    applicationServerKey: urlBase64ToUint8Array(vapidPublicKey),
  });

  // Send this 'subscription' object to your backend to store it
  await fetch('/save-subscription', {
    method: 'POST',
    body: JSON.stringify(subscription),
    headers: { 'Content-Type': 'application/json' },
  });
  console.log('Subscribed to push notifications');
}

// Utility function to convert VAPID key
function urlBase64ToUint8Array(base64String) {
  const padding = '='.repeat((4 - base64String.length % 4) % 4);
  const base64 = (base64String + padding).replace(/-/g, '+').replace(/_/g, '/');
  const rawData = window.atob(base64);
  const outputArray = new Uint8Array(rawData.length);
  for (let i = 0; i < rawData.length; ++i) {
    outputArray[i] = rawData.charCodeAt(i);
  }
  return outputArray;
}
```

### 5. Backend Server (Conceptual)
You need a simple backend (e.g., using Node.js with the `web-push` library) to:
1.  **Generate VAPID keys**: These keys identify your server to the push services.
2.  **Create an endpoint `/save-subscription`**: This endpoint receives the `subscription` object from the client and saves it.
3.  **Create a trigger**: An endpoint or script that sends a push message to the saved subscription URL.

### 6. Testing
1.  **Serve over HTTPS**: PWA features require a secure context. Start a local server and use a tool like **`ngrok`** to create a public HTTPS URL (`ngrok http <your-port>`).
2.  **Access on Device**: Open the `ngrok` HTTPS URL on your Android or iOS device.
3.  **Install the PWA**:
    - **Android (Chrome)**: Look for the "Install app" prompt or use the "Add to Home Screen" option in the menu.
    - **iOS (Safari)**: Use the "Share" button and select "Add to Home Screen".
4.  **Test Push**: Click the "Subscribe" button in the app. Then, trigger the push message from your backend. The notification should appear on your device, even if the app is closed.
5. Close and end the app (also remove it from the background tasks). Then trigger another push message and check if it's sent correctly even from the background.

