// based on https://github.com/serenity-rs/serenity/blob/current/examples/e14_slash_commands/src/main.rs

use std::fs;
use std::time::Instant;

use serenity::async_trait;
use serenity::model::application::command::Command;
use serenity::model::application::interaction::{Interaction, InteractionResponseType};
use serenity::model::gateway::Ready;
use serenity::model::channel::Message;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::id::UserId;
use serenity::prelude::*;

// needed for shutdown command
use lazy_static::lazy_static;
use tokio::sync::mpsc::{Sender, channel};

lazy_static! { static ref SHUTDOWN_SENDER: Mutex<Option<Sender<bool>>> = Mutex::new(None); }

lazy_static! { static ref ADMIN_USERS: Vec<UserId> = vec![UserId(165216105197993984)]; }

struct Handler;

// for some reason if you don't specify the return type the compiler doesn't figure it out
async fn send_interaction_response_message<D>(ctx: &Context, command: &ApplicationCommandInteraction, content: D) -> Result<(), SerenityError> where D: ToString {
    command.create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| message.content(content))
        })
        .await
}

async fn edit_interaction_response_message<D>(ctx: &Context, command: &ApplicationCommandInteraction, content: D) -> Result<Message, SerenityError> where D: ToString {
    command.edit_original_interaction_response(&ctx.http, |response| {
        response.content(content)
    })
    .await
}

async fn nyi_command(ctx: Context, command: ApplicationCommandInteraction) -> Result<(), SerenityError> {
    send_interaction_response_message(&ctx, &command, "This command hasn't been implemented. Try /help").await
}

async fn ping_command(ctx: Context, command: ApplicationCommandInteraction) -> Result<(), SerenityError> {
    let start_time = Instant::now();
    command.defer(&ctx.http).await?;
    let mut duration = start_time.elapsed().as_millis().to_string();
    duration.push_str(" ms");
    edit_interaction_response_message(&ctx, &command, duration).await?;
    Ok(())
}

async fn shutdown_command(ctx: Context, command: ApplicationCommandInteraction) -> Result<(), SerenityError> {
    if !ADMIN_USERS.contains(&command.user.id) {
        send_interaction_response_message(&ctx, &command, "You do not have permission.").await?;
        return Ok(())
    }
    send_interaction_response_message(&ctx, &command, "Shutting down...").await?;
    let lock = SHUTDOWN_SENDER.lock().await;
    let sender = &lock.as_ref().expect("Shutdown command called before shutdown channel initialized??");
    sender.send(true).await.expect("Shutdown message send error");
    println!("Passing shutdown message");
    drop(lock);
    ctx.shard.shutdown_clean();
    Ok(())
}

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {            
            if let Err(why) = match command.data.name.as_str() {
                "ping" => ping_command(ctx, command).await,
                "shutdown" => shutdown_command(ctx, command).await,
                _ => nyi_command(ctx, command).await
            } {
                println!("Cannot respond to slash command: {}", why);
            };
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        Command::set_global_application_commands(&ctx.http, |commands| {
            commands
                .create_application_command(|command| {
                    command.name("ping").description("A ping command")
                })
                .create_application_command(|command| {
                    command.name("shutdown").description("Shut down the bot")
                })
        })
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

    let (sender, mut receiver) = channel(64);
    *SHUTDOWN_SENDER.lock().await = Some(sender);

    let shard_manager = client.shard_manager.clone();

    tokio::spawn(async move {
        loop {
            let b = receiver.recv().await.expect("Shutdown message pass error");
            if b {
                shard_manager.lock().await.shutdown_all().await;
                println!("Shutdown shard manager");
                break;
            }
        }
    });

    println!("Client shutdown: {:?}", client.start().await);
}