/*
 * Copyright (c) Facebook, Inc. and its affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::env;
use std::collections::HashMap;
use std::sync::Arc;

use serenity::async_trait;
use serenity::framework::standard::macros::{command, group, hook};
use serenity::framework::standard::{Args, CommandResult, StandardFramework};
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use serenity::utils::MessageBuilder;
mod gwgp;

const HELP_MESSAGE: &str = "
Hello there, Human!

You have summoned me. Let's see about getting you what you need.

❓ Need technical help?
➡️ Post in the <#1143984898953191495> channel and other humans will assist you.

❓ Something wrong?
➡️ You can flag an admin with @admin

I hope that resolves your issue!

— HelpBot 🤖
";

struct CommandResp;

impl TypeMapKey for CommandResp {
    type Value = gwgp::Resp;
}

#[group]
#[commands(help, get)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    // async fn message(&self, ctx: Context, msg: Message) {}

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN")
        .expect("Expected a token in the environment");

    let framework = StandardFramework::new()
        .configure(|c| c
                   .with_whitespace(true)
                   .prefix("~")
                   .delimiters(vec![", ", ","]))
        .group(&GENERAL_GROUP);

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Err creating client");

    {
        let mut data = client.data.write().await;
        let gwgp_info = gwgp::get_prices().await;
        match gwgp_info {
            Ok(v) => data.insert::<CommandResp>(v),
            Err(why) => println!("Enconter error: {:?}", why)
        };

    }

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}

#[command]
async fn help(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, HELP_MESSAGE).await?;

    Ok(())
}

#[command]
async fn get(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let city = args.single::<String>()?.clone();
    let data = ctx.data.read().await;

    let prices_info = data.get::<CommandResp>().expect("Expected Resp in Typemap");
    let date = prices_info.date_info.clone();
    let selected_city_price = prices_info.prices.get(&city);
    match selected_city_price {
        Some(val) => 
        {
            let response = MessageBuilder::new()
            .push(date)
            .push("\n")
            .push(val)
            .build();
            msg.reply(ctx, response).await?
        }
        None => msg.reply(ctx, "Unable to get the info").await?, 
    };
    

    Ok(()) 
}
