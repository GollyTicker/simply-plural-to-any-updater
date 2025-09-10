# !! UNDER DEVELOPMENT !!
**Project is actively moving from v1 to v2 and v2 isn't there where it wants to be yet**

v1: Simple local CLI to sync SP to VRChat and Discord

v2: Cloud Service where users can register and have SP synced to other platforms via a GUI and where self-hosting isn't necessary.

Once v2 is done, this warnijg will be removed. If you try to use the v2 codebase before, it'll likely not work.

----

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

Build: `./steps/12-backend-cargo-build.sh`

Lint and Format: `./steps/10-lint.sh`

The environment variables are documented in `defaults.env` and `vrcupdater.sample.env`.

All functionality is implemented using Rust and various libraries.

For developers, one can use `/dev/*.run.sh` for local quick running.

Codebase size: `./dev/codebase-size.sh`

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

* add status not only for updaters but also for SP itself.
* Add automatic sync to PluralKit
* complete migration to webapp
* add `enable_website` config
* Rename 'VRChat Status Name' field to 'SP2Any Simple Name' field
* Ask on Reddit and various discord servers for what features the users want
* persistent deployment:
  * cheap free tier VMs and docker based deployment?
  * using [free managed postgres](https://www.bytebase.com/blog/postgres-hosting-options-pricing-comparison/) with free tier serverless functions?
  * alternatively a mixture of the above?
* make it such that the code can ALSO run in a mobile app.
  * UI should be easily adapted to be running in mobile app in additional to a web-app
  * backend would be mostly in the cloud OR locally on the modile.
  * database needs to support both postgres and SQLite, since the database will be different based on mobile vs. cloud
* On UI:
  * add quick and easy feedback field
  * add link to Discord Server
  * add link to KoFi and ask for kind donations
  * add link to source code

#### User Feedback
* sync from and to pluralkit as well (checkout pk-rpc). most SP -> PK
* add a warning, that using the discord self-botting comes with a risk for both the user and the dev
  * [artcle by discord](https://support.discord.com/hc/en-us/articles/115002192352-Automated-User-Accounts-Self-Bots)
  * [self-botting](https://gist.github.com/nomsi/2684f5692cad5b0ceb52e308631859fd)
  * [reddit 1](https://old.reddit.com/r/Discord_selfbots/comments/t9o5xf/anyone_got_banned/), [reddit 2](https://old.reddit.com/r/discordapp/comments/7nl35v/regarding_the_ban_on_selfbots/)
* DONE discord rich presence / activity as instead of discord status
* DONE: create discord server with initial interests
  * share in pluralkit server
  * plural hub server
  * other plural reddit communities
* share with refactionvr server mods before sharing in channel
