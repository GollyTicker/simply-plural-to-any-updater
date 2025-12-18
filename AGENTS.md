# Instructions for AI Coding Agents


## What is PluralSync?

A cloud service where users can automatically sync their plural systems' fronting status
between various system managers and social platforms such as [SimplyPLural](https://apparyllis.com/), [PluralKit](https://pluralkit.me/), [VRChat](https://hello.vrchat.com/), [Discord](https://discord.com) or their own website. Users of system managers (plural systems, DID/OSDD systems, etc.) benefit from this as it makes it easier for them to communicate who's fronting while only
needing to update their fronting on Simply Plural.

A public test version can be found online at [public-test.sp2any.ayake.net](https://public-test.sp2any.ayake.net). (*Use this at your own risk.*)

Currently the following updates are supported:
* SimplyPlural to VRChat Status
* SimplyPlural to Discord Status / Discord Rich Presence
* SimplyPlural to Website with fronter names and avatars
* SimplyPlural to PluralKit Fronters Switch 

We, the developers, take data security and privacy seriously. The data to synchronise between the services
is stored encrypted and at industry-standard security. Self hosting is possible if you have some tech knowledge.

## General DOs

Only do the tasks described when explicitly requested to.

## Coding Guidelines

* Rust import statemnts should be one crate per statement. Importing multiple objects from the same create should be done in the same statement.
  * Good: `use anyhow::{anyhow, Error, Result}`
  * Bad: the above imports on separate lines/statements for each imported object
* Rust import statements should use separate lines for imports from different modules originating from this project.

## Architecture

### Backend (`sp2any`)

*   **Language:** Rust
*   **Framework:** [Rocket](https://rocket.rs/)
*   **Functionality:**
    *   Provides the core backend services, including a RESTful API and WebSocket communication.
    *   Interacts with a PostgreSQL database using `sqlx` for data persistence.
    *   Manages user authentication using JSON Web Tokens (JWT).
    *   Communicates with external services like the VRChat API.
    *   Exposes application metrics for monitoring via Prometheus.
*   **Tooling:**
    *   Includes a utility (`ts-bindings`) to generate TypeScript type definitions from Rust code using `specta`, ensuring type safety between the backend and frontend.

### Web Frontend (`frontend`)

*   **Framework:** [Vue.js](https://vuejs.org/) with TypeScript
*   **Build Tool:** [Vite](https://vitejs.dev/)
*   **Functionality:**
    *   Provides the main user interface for the web application.
    *   Communicates with the Rust backend via HTTP requests (using `axios`) and WebSockets.

### Desktop Application (`sp2any-bridge`)

*   **Framework:** [Tauri](https://tauri.app/) (Rust backend, web-based frontend)
*   **Backend (`bridge-src-tauri`):**
    *   Written in Rust.
    *   Integrates with the operating system for features like autostart.
    *   Includes Discord Rich Presence integration.
    *   Communicates with the main `sp2any` backend.
*   **Frontend (`bridge-frontend`):**
    *   A web-based UI built with TypeScript and Vite.
    *   Uses the Tauri API to interact with the Rust backend part of the desktop application.

### Shared Code (`base-src`)

*   **Language:** Rust
*   **Purpose:**
    *   A shared library containing common data structures, types, and utilities.
    *   This crate is used as a dependency by both the main backend (`sp2any`) and the Tauri backend (`sp2any-bridge`), promoting code reuse and consistency.
