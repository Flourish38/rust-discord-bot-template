# rust-discord-bot-template

This is a fleshed out project to start a discord bot using [serenity](https://github.com/serenity-rs/serenity).

### How to use it

Put your token in a file called `config.(ini|json|yaml|toml|ron|json5)` with the key "token".
You can also specify admin users in an array with the key "admins". By default, this is only used for the shutdown command.

For example, a file `config.toml` would look like:
```toml
token = "TOKEN_GOES_HERE"
admins = [ 123456789876543210 ]
```

If you would rather, you can instead provide your token by the environment variable DISCORD_TOKEN.

Once you decide on a file format for your config file, you can disable the ones you aren't using in `Cargo.toml` according to [config-rs features](https://github.com/mehcode/config-rs#feature-flags).

From there, add more commands in `src/commands.rs`, and implement any necessary components in `src/components.rs`.
You ***shouldn't*** need to modify `src/main.rs` at all, since the config is accessible as a static variable.
The only exception is if you need to use a different type of interaction than commands and components,
which you could simply add to the match statement on line 39 of `src/main.rs`.


Template made by [Flourish38](https://github.com/Flourish38).