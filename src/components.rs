use std::time::Instant;

use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::prelude::interaction::message_component::MessageComponentInteraction;
use serenity::prelude::*;

pub async fn handle_component(ctx: Context, component: MessageComponentInteraction) -> Result<(), SerenityError> {
    // Add any custom components here
    match component.data.custom_id.as_str() {
        "refresh_ping" => ping_refresh_component(ctx, component).await,
        _ => nyi_component(ctx, component).await
    }
}

async fn nyi_component(ctx: Context, component: MessageComponentInteraction) -> Result<(), SerenityError> {
    // This prepends a notice to the message that the component is implemented on.
    // It does not remove the components from the message.
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
    // Use awaiting the defer as a delay to calculate the ping.
    // This gives very inconsistent results, but imo is probably closer to what you want than a heartbeat ping.
    component.defer(&ctx.http).await?;
    let mut duration = start_time.elapsed().as_millis().to_string();
    duration.push_str(" ms");
    // This does not remove the refresh component from the original message.
    component.edit_original_interaction_response(&ctx.http, |response| {
        response.content(duration)
    }).await?;
    Ok(())
}