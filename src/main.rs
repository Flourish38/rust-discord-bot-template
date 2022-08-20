// based on https://github.com/serenity-rs/serenity/blob/current/examples/e14_slash_commands/src/main.rs
// you **shouldn't** need to modify this file at all, unless you want to use an interaction other than commands and components.
// in that case, modify interaction_create below and create a separate module for it in another file.

mod commands;
mod components;

use commands::*;
use components::*;

use std::fs;

use serenity::async_trait;
use serenity::model::application::command::Command;
use serenity::model::application::interaction::Interaction;
use serenity::model::gateway::Ready;
use serenity::model::id::UserId;
use serenity::prelude::*;

use lazy_static::lazy_static;
use tokio::sync::mpsc::channel;

lazy_static! { static ref ADMIN_USERS: Vec<UserId> = vec![UserId(165216105197993984)]; }

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::ApplicationCommand(command) => {
                // Commands are implemented in src/commands.rs
                if let Err(why) = handle_command(ctx, command).await {
                    println!("Cannot respond to slash command: {}", why);
                };
            },
            Interaction::MessageComponent(component) => {
                // Components are implemented in src/components.rs
                if let Err(why) = handle_component(ctx, component).await {
                    println!("Cannot respond to message component: {}", why);
                }
            },
            _ => println!("Unimplemented interaction: {}", interaction.kind().num())
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
        
        Command::set_global_application_commands(&ctx.http, create_commands)
        .await.expect("Failed to set application commands");
    }
}

#[tokio::main]
async fn main() {
    // Configure the client with your Discord bot token in token.txt.
    let token = fs::read_to_string("token.txt").expect("Expected a token in token.txt");

    // Build our client.
    let mut client = Client::builder(token, GatewayIntents::empty())
        .event_handler(Handler)
        .await
        .expect("Error creating client");

    // Channel for the shutdown command to use later
    let (sender, mut receiver) = channel(64);
    *SHUTDOWN_SENDER.lock().await = Some(sender);

    let shard_manager = client.shard_manager.clone();

    // Spawns a task that waits for the shutdown command, then shuts down the bot.
    tokio::spawn(async move {
        loop {
            // I have left open the possibility of using b=false for something "softer" in case you need it.
            let b = receiver.recv().await.expect("Shutdown message pass error");
            if b {
                shard_manager.lock().await.shutdown_all().await;
                println!("Shutdown shard manager");
                break;
            }
        }
    });

    // Start the client.
    println!("Client shutdown: {:?}", client.start().await);
}