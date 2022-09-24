# Nuko Bot

Nuko a simple bot for specific needs

Designed for compiling Rust to WebAssembly and publishing the resulting worker to 
Cloudflare's [edge infrastructure](https://www.cloudflare.com/network/).

## Supporteed and planned commands

- [ ] `xe`: Currency exchange
- [ ] `xe <~set_defaults> <~amount> <~from> <~to>`, For example `xe 1 AUD JPY` is equivalent to `xe 1` with defaults
- [x] `jisho`: Search jisho.org
- [x] `jisho <word>`: Search jisho.org for a word

## Adding new commands

To add a new command simply implement the `Command` trait. For example to add a ping command

1. create a file src/commands/ping.rs

``` rust
use crate::interaction::{
    InteractionApplicationCommandCallbackData, ApplicationCommandOption, ApplicationCommandOptionChoice, ApplicationCommandInteractionDataOption, ApplicationCommandOptionType
};
use crate::error::InteractionError;
use crate::command::Command;

use async_trait::async_trait;


pub(crate) struct Ping {}

#[async_trait(?Send)]
impl Command for Ping {
    async fn respond(&self, _options: &Option<Vec<ApplicationCommandInteractionDataOption>>, _ctx: &mut worker::RouteContext<()>) -> Result<InteractionApplicationCommandCallbackData, InteractionError>{
        Ok(InteractionApplicationCommandCallbackData {
            content: Some("Pong".to_string()),
            choices: None,
            embeds: None
        })
    }

    fn name(&self) -> String{
        "ping".into()
    }

    fn description(&self) -> String {
        "Send a ping".into()
    }

    fn options(&self) -> Option<Vec<ApplicationCommandOption>> {
        // add any arguments/choices here, more info at https://discord.com/developers/docs/interactions/application-commands#application-command-object-application-command-option-structure
        None
    }

    async fn autocomplete(&self, _options: &Option<Vec<ApplicationCommandInteractionDataOption>>, _ctx: &mut worker::RouteContext<()>) -> 
        None
    }

```
2. add your new module in src/commands/mod.rs
3. Register your command in  `init_commands` in src/command.rs 
``` rust
pub(crate) fn init_commands() -> Vec<Box<dyn Command + Sync>> {
    let mut v : Vec<Box<dyn Command + Sync>> = Vec::new();
    v.push(Box::new(commands::hello::Hello {}));
    // Add this line
    v.push(Box::new(commands::ping::Ping {}));
    v
}
```
4. publish your package with `wrangler publish`
5. register your new command with discord with `curl -X POST http://bot.<mydomain>.workers.dev/register`

You can store and access state using the `ctx` context object passed to the `respond` and `autocomplete` methods, for example:

``` rust
let kv = ctx.kv("my_namespace")?;  // the namespace must be first registered on cloudflare dashboard
let my_val =  kv.get("my_key").text().await?;
kv.put("foo", "bar")?.execute().await?;

```

## Local Dev 


With `wrangler`, you can build, test, and deploy your Worker with the following commands: 

```bash
# compiles your project to WebAssembly and will warn of any issues
wrangler build 

# run your Worker in an ideal development workflow (with a local server, file watcher & more)
wrangler dev

# deploy your Worker globally to the Cloudflare network (update your wrangler.toml file for configuration)
wrangler publish
```

you can use `ngrok` to tunnel traffic into your local machine, more info [here](https://discord.com/developers/docs/tutorials/hosting-on-cloudflare-workers#setting-up-ngrok)

## Credits

based on [stateless-discord-bot](https://github.com/siketyan/stateless-discord-bot)
[pure-rust-discord-bot]()