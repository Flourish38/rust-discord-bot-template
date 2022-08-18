// based on https://github.com/serenity-rs/serenity/blob/current/examples/e14_slash_commands/src/main.rs

use std::fs;
use std::time::{Instant, Duration};

use tokio::time::sleep;

use serenity::async_trait;
use serenity::model::application::command::Command;
use serenity::model::application::interaction::{Interaction, InteractionResponseType};
use serenity::model::gateway::Ready;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::id::UserId;
use serenity::model::prelude::component::ButtonStyle;
use serenity::model::prelude::interaction::message_component::MessageComponentInteraction;
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

async fn handle_command(ctx: Context, command:ApplicationCommandInteraction) -> Result<(), SerenityError> {
    match command.data.name.as_str() {
        "help" => help_command(ctx, command).await,
        "ping" => ping_command(ctx, command).await,
        "shutdown" => shutdown_command(ctx, command).await,
        _ => nyi_command(ctx, command).await
    }
}

async fn nyi_command(ctx: Context, command: ApplicationCommandInteraction) -> Result<(), SerenityError> {
    send_interaction_response_message(&ctx, &command, "This command hasn't been implemented. Try /help").await
}

async fn help_command(ctx: Context, command: ApplicationCommandInteraction) -> Result<(), SerenityError> {
    command.create_interaction_response(&ctx.http, |response| {
        response.kind(InteractionResponseType::ChannelMessageWithSource)
            .interaction_response_data(|data| {
                data.ephemeral(true)
                    .content("Currently available commands: `/ping`, `/shutdown`, `/help`.")
            })
    }).await
    // for some reason you can't delete ephemeral interaction responses so I guess I'll just suffer
}

async fn ping_command(ctx: Context, command: ApplicationCommandInteraction) -> Result<(), SerenityError> {
    let start_time = Instant::now();
    command.defer(&ctx.http).await?;
    let mut duration = start_time.elapsed().as_millis().to_string();
    duration.push_str(" ms");
    command.edit_original_interaction_response(&ctx.http, |response| {
        response.content(duration)
        .components(|components| {
            components
                .create_action_row(|action_row| {
                    action_row.create_button(|button| {
                        button.style(ButtonStyle::Secondary)
                            .emoji('🔄')
                            .custom_id("refresh_ping")
                    })
                })
        })
    }).await?;
    Ok(())
}

async fn shutdown_command(ctx: Context, command: ApplicationCommandInteraction) -> Result<(), SerenityError> {
    if !ADMIN_USERS.contains(&command.user.id) {
        send_interaction_response_message(&ctx, &command, "You do not have permission.").await?;
        sleep(Duration::from_secs(5)).await;
        command.delete_original_interaction_response(&ctx.http).await?;
        return Ok(())
    }
    send_interaction_response_message(&ctx, &command, "Shutting down...").await?;
    let lock = SHUTDOWN_SENDER.lock().await;
    let sender = &lock.as_ref().expect("Shutdown command called before shutdown channel initialized??");
    sender.send(true).await.expect("Shutdown message send error");
    drop(lock);
    println!("Passing shutdown message");
    ctx.shard.shutdown_clean();
    Ok(())
}

async fn handle_component(ctx: Context, component: MessageComponentInteraction) -> Result<(), SerenityError> {
    match component.data.custom_id.as_str() {
        "refresh_ping" => ping_refresh_component(ctx, component).await,
        _ => nyi_component(ctx, component).await
    }
}

async fn nyi_component(ctx: Context, component: MessageComponentInteraction) -> Result<(), SerenityError> {
    let mut content = "Component interaction not yet implemented.\n".to_string();
    content.push_str(&component.message.content);
    component.create_interaction_response(&ctx.http, |response| {
        response.kind(InteractionResponseType::UpdateMessage)
            .interaction_response_data(|data| {
                data.content(content)
            })
    })
    .await?;
    Ok(())
}

async fn ping_refresh_component(ctx: Context, component: MessageComponentInteraction) -> Result<(), SerenityError> {
    let start_time = Instant::now();
    component.defer(&ctx.http).await?;
    let mut duration = start_time.elapsed().as_millis().to_string();
    duration.push_str(" ms");
    component.edit_original_interaction_response(&ctx.http, |response| {
        response.content(duration)
    }).await?;
    Ok(())
}

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::ApplicationCommand(command) => {
                if let Err(why) = handle_command(ctx, command).await {
                    println!("Cannot respond to slash command: {}", why);
                };
            },
            Interaction::MessageComponent(component) => {
                if let Err(why) = handle_component(ctx, component).await {
                    println!("Cannot respond to message component: {}", why);
                }
            },
            _ => println!("Unimplemented interaction: {}", interaction.kind().num())
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        Command::set_global_application_commands(&ctx.http, |commands| {
            commands
                .create_application_command(|command| {
                    command.name("help").description("Information on how to use the bot")
                })
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

    // Channel for the shutdown command to use later
    let (sender, mut receiver) = channel(64);
    *SHUTDOWN_SENDER.lock().await = Some(sender);

    let shard_manager = client.shard_manager.clone();

    // Spawns a task that waits for the shutdown command, then shuts down the bot.
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

    // Start the client.
    println!("Client shutdown: {:?}", client.start().await);
}