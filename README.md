**Project is actively under development. Most things work well however.**

----

# PluralSync - Sync your plural system across platforms

A cloud service where users can automatically sync their plural systems' fronting status
between various system managers and social platforms such as [SimplyPLural](https://apparyllis.com/), [PluralKit](https://pluralkit.me/), [VRChat](https://hello.vrchat.com/), [Discord](https://discord.com) or their own website. Users of system managers (plural systems, DID/OSDD systems, etc.) benefit from this as it makes it easier for them to communicate who's fronting while only
needing to update their fronting on Simply Plural.

A public test version can be found online at [public-test.pluralsync.ayake.net](https://public-test.pluralsync.ayake.net). (*Use this at your own risk.*)

Currently the following updates are supported:
* SimplyPlural to VRChat Status
* SimplyPlural to Discord Status / Discord Rich Presence
* SimplyPlural to Website with fronter names and avatars
* SimplyPlural to PluralKit Fronters Switch 

We, the developers, take data security and privacy seriously. The data to synchronise between the services
is stored encrypted and at industry-standard security. Self hosting is possible if you have some tech knowledge.

Developed with ❤️ by [Ayake](https://github.com/GollyTicker)\*.

## For Developers

Prerequisites:
* Rust toolchain (ideally via rustup)
* node + npm
* `./steps/02-install-dependencies.sh`

Build: `./steps/12-backend-cargo-build.sh`

Lint and Format: `./steps/10-lint.sh`

Deployment environment variables are currently undocumented. But you can checkout `docker/local.env` as a starting point.

Codebase size: `./dev/codebase-size.sh`

And run the files in `test` for testing. For the integration tests,
you'll need to export the `SPS_API_TOKEN` and `SPS_API_WRITE_TOKEN` of the plural system used for tests - 
as well as `VRCHAT_USERNAME`, `VRCHAT_PASSWORD` and `VRCHAT_COOKIE` of the VRC test user etc.

To create a release, make a new tag (e.g. `v2.10`) and run `./steps/32-publish-release.sh`.

Check dependencies bloat via `cargo bloat --release --bin pluralsync`.
