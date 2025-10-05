
# First feedback post-deployment
**Priority**
* member privacy
  * DONE custom fronts configuration
  * DONE archived members configuration / hiding
  * make lists of members / CFs / archived members collapeble and searchable to manage large systems
  * probably start with defaults for actives / archived / CFs and integration with privacy buckets from SP
    * [privacy buckets in SimplyPlual](https://docs.apparyllis.com/docs/help/features/buckets/intro). Perhaps
      we can also instead make a singleton "SP2Any" account on SP and people can add that one as a friend.
      This way, they can simply assign SP2Any to existing privacy bucket groups and chose what should be shown.
      This is an alternative to asking the users to make a new privacy bucket with the name "SP2Any" which is then read by the API.
    * privacy bucket API doesn't seem to be documented. I'll have to reverse-engineer that.
      * https://api.apparyllis.com/v1/privacyBuckets/
  * bidrectional sync of privacy bucket membership and "show in SP2Any" setting
    > If I search for myself, and toggle the "show as fronting" button in SP2A, it autoadds me to the privacy bucket in SP.
    > And if I add myself to the PB in SP, it toggles me as "show as fronting"
* websocket connection restarts
* better error messages which the users can also understand and which handle most common error paths

Privacy buckets of each member/custom front is simply a list of bucket-ids.

Privacy buckets API:
GET https://api.apparyllis.com/v1/privacyBuckets/
returns
[
  {
    id: string, // bucket uid
    content: {
      uid: string, // system-uid
      name: string,
      icon: string,
      desc: string,
      rank: string, // what meaning?
      color: "#123456",
      lastOperationTime: number,
    }
  },
  ...
]

* registration logs in by default as well automatically
* security: make it such that on my private instance, only handpicked users may register and use it.
* configureable order in which fronts are shown
* make it more clear, what the people need to do make the discord bridge thing work. maybe a list of steps and if they're working.
* support large systems. i.e. members search and bulk edit.

* configs changing didnt quite work 
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
* reduce compile times by removing vrchatapi library and using http rest requests directly
* better split sp2any crate into what is exported to bridge-src-tauri and what is not. makes for much faster compiles
* merge cargo crates into a single workspace to improve build times
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

---

# First deployment
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
