use crate::ADMIN_USERS;

use std::time::Instant;

use serenity::builder::CreateApplicationCommands;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::component::ButtonStyle;
use serenity::prelude::*;

// needed for shutdown command
use lazy_static::lazy_static;
use tokio::sync::mpsc::Sender;

lazy_static! { pub static ref SHUTDOWN_SENDER: Mutex<Option<Sender<bool>>> = Mutex::new(None); }

// for some reason if you don't specify the return type the compiler doesn't figure it out
async fn send_interaction_response_message<D>(ctx: &Context, command: &ApplicationCommandInteraction, content: D, ephemeral: bool) -> Result<(), SerenityError> where D: ToString {
    command.create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| message.content(content).ephemeral(ephemeral))
        })
        .await
}

pub fn create_commands(commands: &mut CreateApplicationCommands) -> &mut CreateApplicationCommands {
    // DON'T FORGET to add your custom commands here!!
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
}
// Any custom slash commands must be added both to create_commands ^^^ and to handle_command!!
pub async fn handle_command(ctx: Context, command:ApplicationCommandInteraction) -> Result<(), SerenityError> {
    // Add any custom commands here
    match command.data.name.as_str() {
        "help" => help_command(ctx, command).await,
        "ping" => ping_command(ctx, command).await,
        "shutdown" => shutdown_command(ctx, command).await,
        _ => nyi_command(ctx, command).await
    }
}

async fn nyi_command(ctx: Context, command: ApplicationCommandInteraction) -> Result<(), SerenityError> {
    send_interaction_response_message(&ctx, &command, "This command hasn't been implemented. Try /help", true).await
}

async fn help_command(ctx: Context, command: ApplicationCommandInteraction) -> Result<(), SerenityError> {
    // This is very bare-bones, you will want to improve it most likely
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
    // Use awaiting the message as a delay to calculate the ping.
    // This gives very inconsistent results, but imo is probably closer to what you want than a heartbeat ping.
    command.create_interaction_response(&ctx.http, |response| {
        response.kind(InteractionResponseType::DeferredChannelMessageWithSource)
            .interaction_response_data(|data| data.ephemeral(true))
    }).await?;
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
    // Set your admin user list in your config file
    let admins = ADMIN_USERS.lock().await;
    if !admins.is_empty() && !admins.contains(&command.user.id) {
        send_interaction_response_message(&ctx, &command, "You do not have permission.", true).await?;
        return Ok(())
    }
    println!("Shutdown from user {} with Id {}", command.user.name, command.user.id);
    // no ? here, we don't want to return early if this fails
    _ = send_interaction_response_message(&ctx, &command, "Shutting down...", true).await;
    // loosely based on https://stackoverflow.com/a/65456463
    // keep the lock separate because you have to (which is neat)
    let lock = SHUTDOWN_SENDER.lock().await;
    // This error means that the shutdown channel is somehow not good, so we actually want to panic
    let sender = lock.as_ref().expect("Shutdown command called before shutdown channel initialized??");
    // If this errors, the receiver could not receive the message anyways, so we want to panic
    sender.send(true).await.expect("Shutdown message send error");
    println!("Passed shutdown message");
    // I'm pretty sure this is unnecessary but it makes me happier than not doing it
    ctx.shard.shutdown_clean();
    Ok(())
}