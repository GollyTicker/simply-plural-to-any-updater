
# First deployment
* test that workflows work with deployed dev-online
* deploy for discord test server users
* security: make it such that on my private instance, only handpicked users may register and use it.
* DONE: make SP2ANY_BASE_URL configureable for frontend-dist
* DONE: deploy on private space once and share with friend
* DONE: add hint on all deployments where it warns about which variant one is on on the nav bar
* DONE: add variant picker for sp2any-bridge, such that it even knows where to connect to!
* DONE: add download link to bridge frontend in UI
* DONE: add `enable_website` config

* add status not only for updaters but also for SP itself.
* remove `0.1.0` from sp2any bridge executable
* Add automatic sync to PluralKit
* Rename 'VRChat Status Name' field to 'SP2Any Simple Name' field
  * ask users on how to best configure this
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
* make sure, that during production, only my own domains are allowed and not localhost or so.
* make website view such that it doesn't eagery fetch data from simply plural every time but instead uses the latest values from a channel
* DONE: complete migration to webapp

* add initial suggestions by ChatAI (e.g. privacy, configurability, etc.)

# User Feedback
* sync from and to pluralkit as well (checkout pk-rpc). most SP -> PK
* add a warning, that using the discord self-botting comes with a risk for both the user and the dev
  * [artcle by discord](https://support.discord.com/hc/en-us/articles/115002192352-Automated-User-Accounts-Self-Bots)
  * [self-botting](https://gist.github.com/nomsi/2684f5692cad5b0ceb52e308631859fd)
  * [reddit 1](https://old.reddit.com/r/Discord_selfbots/comments/t9o5xf/anyone_got_banned/), [reddit 2](https://old.reddit.com/r/discordapp/comments/7nl35v/regarding_the_ban_on_selfbots/)
  * perhaps use the same approach as used by the discord chat exporter? this might actually work well.
* share with refactionvr server mods before sharing in channel

