# TASK

We want to make the SP2ANY_BASE_URL backend used by the sp2any bridge-frontend configureable in the UI. It currently has a default base url hard-coded. As an end result, we want to extend the existing variant-info placeholder (index.html in bridge-frontend). This base url shall be configureable during in the login form and be submitted together with the login request. Don't add the input fields to the #variant-info. That #variant-info is only for display!

Take a look at the source code files and make a plan for the steps to implement this. Write down the steps below but don't execute any steps.

## STEPS

1.  **Analyze `bridge-frontend/index.html`**: Examine the structure of the login form and the `#variant-info` element to identify where to add the new input field and display the configured URL.
2.  **Analyze `bridge-frontend/src/main.ts`**: Investigate the TypeScript code that handles the login logic. Understand how the form is currently submitted and where the hard-coded `SP2ANY_BASE_URL` is used in `bridge-frontend/**`.
3.  **Modify `bridge-frontend/index.html`**:
    *   Add a new `<input>` field to the login form for the `SP2ANY_BASE_URL`. It should have a proper label and an ID (e.g., `sp2any-base-url`).
    *   Add a new element (e.g., a `<span>` with an ID like `configured-base-url`) inside the `#variant-info` div to display the configured base URL.
4.  **Modify `bridge-frontend/src/main.ts`**:
    *   Update the login form's event listener.
    *   Inside the listener, retrieve the value from the new `sp2any-base-url` input field.
    *   Use this value to dynamically set the base URL for API calls.
    *   After a successful login, store the configured base URL in the browser's local storage to persist it.
    *   On page load, check for the base URL in local storage and, if it exists, populate both the input field and the display element in `#variant-info`.
