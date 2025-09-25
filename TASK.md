# TASK

Add logic to the `frontend` such that any `401` and `403` Responses from the backend make one automatically log-out (delete the local Jwt) and forward to `Start.vue` where the login is re-done.

## STEPS

- In `frontend/src/sp2any_api.ts`, add a response interceptor to the `sp2any_api` axios instance.
- The interceptor should check for `401` or `403` status codes in error responses.
- If such a status code is found, the interceptor will:
    - Remove the `jwt` from `localStorage`.
    - Redirect the user to the `Start` page using the Vue router.
- Ensure the rejected promise is still returned from the interceptor.

