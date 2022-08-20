use std::time::Instant;

use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::prelude::interaction::message_component::MessageComponentInteraction;
use serenity::prelude::*;

pub async fn handle_component(ctx: Context, component: MessageComponentInteraction) -> Result<(), SerenityError> {
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