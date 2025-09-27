# TASK

We want to make the SP2ANY_BASE_URL backend used by the sp2any bridge-frontend configureable in the UI. It currently has a defualt base url hard-coded. As an end result, we want to extend the existing variant-info placeholder and allow users to enter a base-url there, if they want to use a non-default base url.

The bridge-frontend should only communiate with the tauri backend and the tauri rust backend should make the HTTP requests against the new configured URL etc.

## STEPS

1.  **`bridge-frontend/index.html`**:
    *   Add an `<input type="text" id="base-url-input">` and a `<button id="save-base-url-button">` inside the `#variant-info` paragraph.

2.  **`bridge-frontend/src/main.ts` (or a new file):**
    *   Add an event listener to the "Save" button.
    *   On click, get the value from the input field.
    *   Call a new Tauri command `set_base_url` with the new URL.
    *   Store the URL in `localStorage` to persist it across sessions.
    *   On startup, read the URL from `localStorage` and send it to the backend using `set_base_url`.

3.  **`bridge-src-tauri/src/lib.rs`**:
    *   Create a new state to hold the base URL: `Arc<Mutex<Option<String>>>`.
    *   Add the new state to the `tauri::Builder` using `.manage()`.
    *   Create a new command `#[tauri::command] async fn set_base_url(base_url: String, state: tauri::State<'_, Arc<Mutex<Option<String>>>>)`. This command will update the state.
    *   Modify `login_anyhow` and `subscribe_to_bridge_channel_anyhow` to take the `state` as an argument.
    *   Inside these functions, get the URL from the state. If it's `None`, use the default URL.
