# simply-plural-to-any-updater

Update your [Simply Plural](https://apparyllis.com/) system fronting status automatically to
* your [VRChat](https://hello.vrchat.com/) status message
* yout [Discord](https://discord.com) custom status message
* your website as HTML

## SimplyPlural to VRChat Status

When running locally as a VRChat-Updater, it'll check the fronting status
on SimplyPlural periodically and update the VRChat status to reflect the current fronts
e.g. `F: Alice, Bob, Claire`.

For this, simply [download the latest executable](https://github.com/GollyTicker/simply-plural-to-any-updater/releases/latest) and run it locally. It'll create an empty file and ask you to put in your SimplyPlural and VRChat credentials.
These credentials are necessary for it to do it's job. After writing the credentials,
run the executable again. It will first login into VRChat. You may need to provide
a 2FA code, if you hav configured one for your account. Then it'll automatically
update your status in VRChat priodically from SimplyPlural. The login is saved in a cookie,
so you won't need to input your 2FA code that often.

## SimplyPlural to Discord

Similarly to above, the fronting status will be reflected in your discord custom status message.
Since Discord supports emojis and a vast space of unicode characters in the status message (in contrast to VRChat),
the member names will not be cleaned like they are done so for VRChat. If a preferred status name is configured in Simply Plural,
then that is used as well.

## SimplyPlural to Website

When running as a website via `--webserver`, it serves an endpoint `/fronting`
and provides a HTML page with the current fronting status (from SimplyPlural)
as a well-rendered UI.

To run the webserver (Linux only):
1. Download the binary from the latest release
2. Populate `sp2any.json` with the relevant variables. Use `release/config/example.json` as guideline for the format and contents.
3. Run the dockerized setup via `docker compose up -d`.

Now on `http://localhost:8000/fronting` you can GET the fronting status.

Use the the deployment example files as guidelines to your custom deployment setup.

## FAQ

**Why is my member name not shown correctly in VRChat?**

VRChat has limitations on what it allows one to show in the VRChat Status message.
While most european letters and accents are supported, special things such as emojis are not.
Hence this tool removes them before forwarding them to VRChat. If you think something is being removed,
while it's actually possible in the VRChat status, then shortly contact me and let me know (or write an issue).

Furthermore, if a member has a name which cannot be represented at all, e.g. `💖⭐`, then you can define a new
custom field in your Simply Plural named `VRChat Status Name` and fill in a VRChat compatible name in that field,
e.g. `Sparkle Star`. This way you can keep on using the proper name in Simply Plural while also having
something readable in VRChat.

Further note, that even if your status is updated from this program, the _menu in VRChat won't update for **you** (this is a a bug in VRChat...)_.
Others will see the new fronting status message - and you can always check the website, that your status message is indeed updated.

## Migrate from v1 to v2

There are a few breaking changes in how to run this program:
* `SERVE_API` is removed. If it was `true`, then instead invoke the program with `--webserver`. Otherwise don't use this argument.
* The program now opens a GUI by default. If you want to keep on using the console only (old behavior), invoke the program with `--no-gui`.

## For Developers

Prerequisites:
* Rust toolchain (ideally via rustup)
* `cargo install sqlx-cli`

Build: `./release/cargo-build.sh`

Lint and Format: `./release/lint.sh`

The environment variables are documented in `defaults.env` and `vrcupdater.sample.env`.

All functionality is implemented using Rust and various libraries.

For developers, one can use `/dev/*.run.sh` for local quick running.

And run the files in `test` for testing. For the integration tests,
you'll need to export the `SPS_API_TOKEN` and `SPS_API_WRITE_TOKEN` of the plural system used for tests - 
as well as `VRCHAT_USERNAME`, `VRCHAT_PASSWORD` and `VRCHAT_COOKIE` of the VRC test user.

To create a release, simply push a corresponding tag - e.g. `v1.2.3`.

Use `--config <filepath>` to specify an alternate directory where the config is stored and retrieved from.

Check dependencies bloat via `cargo bloat --release --bin sp2any`.

Use the following prompt against the code agent to put it to work:
```
Ensure the project adheres to the coding guidelines.
```
or
```
Update the dependencies.
```

---

## TODOs

### Typesafe API calls

Here is a concise summary of the specta approach for automatically keeping your Rust and TypeScript types consistent.

The strategy is to make your Rust code the single source of truth for any data structures shared with the frontend.

* Annotate Rust Types: In your Rust code, you find the structs and enums that are sent to the frontend (like LocalJsonConfigV2 and UpdaterState). You then add #[derive(specta::Type)] to them.
* Add a Build Script: You create a build.rs file in your project's root. This script uses the specta-typescript library to find all the types you annotated.
* Generate TypeScript: When you compile your Rust project with cargo build, the build script automatically runs. It generates a TypeScript file (e.g., frontend/src/bindings.ts) containing the TypeScript equivalent of your Rust types.
* Use in Frontend: In your frontend code, you import the types from this auto-generated bindings.ts file. This allows you to use them in your fetch requests and component logic, guaranteeing that the frontend's understanding of the data structure always matches the backend's definition.

### TODO

* Rename 'VRChat Status Name' field to 'SP2Any Simple Name' field
* Ask on Reddit and various discord servers for what features the users want

#### User Feedback
* discord rich presence / activity as additonal option instead of status only

