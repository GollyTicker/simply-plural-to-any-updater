# !! UNDER DEVELOPMENT !!

**Project is actively under development. Some things work. Some don't.**

----

# SP2Any - Simply Plural to *Any* Updater

A cloud service where users can automatically sync their [Simply Plural](https://apparyllis.com/) fronting status
to various social platforms such as [VRChat](https://hello.vrchat.com/), [Discord](https://discord.com) or their own website. Users of SimplyPlural (plural systems, DID/OSDD systems, etc.) benefit from this as it makes it easier for them to communicate who's fronting while only
needing to update their fronting on Simply Plural.

An unstable / public test version can be found online at [SP2Any](https://public-test.sp2any.ayake.net). (*Use this at your own risk.*)

We, the developers, take data security and privacy seriously. The data to synchronise between the services
is stored encrypted and at industry-standard security. Additionally, we're planning to add an local app-based version
where the data is stored on the smartphone only and not on our servers. Self hosting is possible if you have some tech knowledge.

Developed with ❤️ by Ayake\*.

* TODO: Move explanations to Config.vue

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

## For Developers

Prerequisites:
* Rust toolchain (ideally via rustup)
* node + npm
* `./steps/03-install-dependencies.sh`

Build: `./steps/12-backend-cargo-build.sh`

Lint and Format: `./steps/10-lint.sh`

Deployment environment variables are currently undocumented. But you can checkout `docker/local.env` as a starting point.

Codebase size: `./dev/codebase-size.sh`

And run the files in `test` for testing. For the integration tests,
you'll need to export the `SPS_API_TOKEN` and `SPS_API_WRITE_TOKEN` of the plural system used for tests - 
as well as `VRCHAT_USERNAME`, `VRCHAT_PASSWORD` and `VRCHAT_COOKIE` of the VRC test user etc.

To create a release, simply push a corresponding tag - e.g. `v1.2.3`.

Check dependencies bloat via `cargo bloat --release --bin sp2any`.

`v1` was the previous version of this tool. It didn't have any UI but was a simple CLI-based tool to run a small server which would sync SP status to VRChat as well as serve a website with the fronting.

