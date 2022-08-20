use crate::ADMIN_USERS;

use std::time::{Instant, Duration};

use tokio::time::sleep;

use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::component::ButtonStyle;
use serenity::prelude::*;

// needed for shutdown command
use lazy_static::lazy_static;
use tokio::sync::mpsc::Sender;

lazy_static! { pub static ref SHUTDOWN_SENDER: Mutex<Option<Sender<bool>>> = Mutex::new(None); }

// for some reason if you don't specify the return type the compiler doesn't figure it out
async fn send_interaction_response_message<D>(ctx: &Context, command: &ApplicationCommandInteraction, content: D) -> Result<(), SerenityError> where D: ToString {
    command.create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| message.content(content))
        })
        .await
}

pub async fn handle_command(ctx: Context, command:ApplicationCommandInteraction) -> Result<(), SerenityError> {
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