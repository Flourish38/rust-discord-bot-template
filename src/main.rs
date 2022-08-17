use std::fs;

use serenity::async_trait;
use serenity::model::application::command::{Command, CommandOptionType};
use serenity::model::application::interaction::application_command::CommandDataOptionValue;
use serenity::model::application::interaction::{Interaction, InteractionResponseType};
use serenity::model::gateway::Ready;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::prelude::*;

// needed for shutdown command
use lazy_static::lazy_static;
use std::sync::mpsc::{Sender, channel};

lazy_static! { static ref SHUTDOWN_SENDER: Mutex<Option<Sender<bool>>> = Mutex::new(None); }

struct Handler;

// for some reason if you don't specify the return type the compiler doesn't figure it out
async fn send_interaction_response_message(ctx: &Context, command: &ApplicationCommandInteraction, content: &String) -> Result<(), SerenityError> {
    command.create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| message.content(content))
        })
        .await
}

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            // println!("Received command interaction: {:#?}", command);

            let content = match command.data.name.as_str() {
                "ping" => "Hey, I'm alive!".to_string(),
                "shutdown" => {
                    let lock = SHUTDOWN_SENDER.lock().await;
                    let sender = &lock.as_ref().expect("Shutdown command called before shutdown channel initialized??");
                    sender.send(true).expect("Shutdown message send error");
                    drop(lock);
                    println!("Passing shutdown message");
                    "Shutting down...".to_string()
                },
                "id" => {
                    let options = command
                        .data
                        .options
                        .get(0)
                        .expect("Expected user option")
                        .resolved
                        .as_ref()
                        .expect("Expected user object");

                    if let CommandDataOptionValue::User(user, _member) = options {
                        format!("{}'s id is {}", user.tag(), user.id)
                    } else {
                        "Please provide a valid user".to_string()
                    }
                },
                "attachmentinput" => {
                    let options = command
                        .data
                        .options
                        .get(0)
                        .expect("Expected attachment option")
                        .resolved
                        .as_ref()
                        .expect("Expected attachment object");

                    if let CommandDataOptionValue::Attachment(attachment) = options {
                        format!(
                            "Attachment name: {}, attachment size: {}",
                            attachment.filename, attachment.size
                        )
                    } else {
                        "Please provide a valid attachment".to_string()
                    }
                },
                _ => "not implemented :(".to_string(),
            };

            if let Err(why) = send_interaction_response_message(&ctx, &command, &content).await
            {
                println!("Cannot respond to slash command: {}", why);
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        let _commands = Command::set_global_application_commands(&ctx.http, |commands| {
            commands
                .create_application_command(|command| {
                    command.name("ping").description("A ping command")
                })
                .create_application_command(|command| {
                    command.name("shutdown").description("Shut down the bot")
                })
                .create_application_command(|command| {
                    command.name("id").description("Get a user id").create_option(|option| {
                        option
                            .name("id")
                            .description("The user to lookup")
                            .kind(CommandOptionType::User)
                            .required(true)
                    })
                })
                .create_application_command(|command| {
                    command
                        .name("welcome")
                        .name_localized("de", "begrüßen")
                        .description("Welcome a user")
                        .description_localized("de", "Einen Nutzer begrüßen")
                        .create_option(|option| {
                            option
                                .name("user")
                                .name_localized("de", "nutzer")
                                .description("The user to welcome")
                                .description_localized("de", "Der zu begrüßende Nutzer")
                                .kind(CommandOptionType::User)
                                .required(true)
                        })
                        .create_option(|option| {
                            option
                                .name("message")
                                .name_localized("de", "nachricht")
                                .description("The message to send")
                                .description_localized("de", "Die versendete Nachricht")
                                .kind(CommandOptionType::String)
                                .required(true)
                                .add_string_choice_localized(
                                    "Welcome to our cool server! Ask me if you need help",
                                    "pizza",
                                    [("de", "Willkommen auf unserem coolen Server! Frag mich, falls du Hilfe brauchst")]
                                )
                                .add_string_choice_localized(
                                    "Hey, do you want a coffee?",
                                    "coffee",
                                    [("de", "Hey, willst du einen Kaffee?")],
                                )
                                .add_string_choice_localized(
                                    "Welcome to the club, you're now a good person. Well, I hope.",
                                    "club",
                                    [("de", "Willkommen im Club, du bist jetzt ein guter Mensch. Naja, hoffentlich.")],
                                )
                                .add_string_choice_localized(
                                    "I hope that you brought a controller to play together!",
                                    "game",
                                    [("de", "Ich hoffe du hast einen Controller zum Spielen mitgebracht!")],
                                )
                        })
                })
                .create_application_command(|command| {
                    command
                        .name("numberinput")
                        .description("Test command for number input")
                        .create_option(|option| {
                            option
                                .name("int")
                                .description("An integer from 5 to 10")
                                .kind(CommandOptionType::Integer)
                                .min_int_value(5)
                                .max_int_value(10)
                                .required(true)
                        })
                        .create_option(|option| {
                            option
                                .name("number")
                                .description("A float from -3.3 to 234.5")
                                .kind(CommandOptionType::Number)
                                .min_number_value(-3.3)
                                .max_number_value(234.5)
                                .required(true)
                        })
                })
                .create_application_command(|command| {
                    command
                        .name("attachmentinput")
                        .description("Test command for attachment input")
                        .create_option(|option| {
                            option
                                .name("attachment")
                                .description("A file")
                                .kind(CommandOptionType::Attachment)
                                .required(true)
                        })
                })
        })
        .await;

        // println!("I now have the following guild slash commands: {:#?}", commands);

        let _guild_command = Command::create_global_application_command(&ctx.http, |command| {
            command.name("wonderful_command").description("An amazing command")
        })
        .await;

        // println!("I created the following global slash command: {:#?}", guild_command);
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

    let (sender, receiver) = channel();
    *SHUTDOWN_SENDER.lock().await = Some(sender);

    let shard_manager = client.shard_manager.clone();

    tokio::spawn(async move {
        loop {
            let b = receiver.recv().expect("Shutdown message pass error");
            if b {
                shard_manager.lock().await.shutdown_all().await;
                println!("Shutdown shard manager");
                break;
            }
        }
    });

    println!("Client shutdown: {:?}", client.start().await);
}