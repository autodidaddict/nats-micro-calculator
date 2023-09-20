use std::{collections::HashMap, sync::RwLock};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use wasmbus_rpc::actor::prelude::*;
use wasmcloud_interface_logging::error;
use wasmcloud_interface_messaging::*;

#[derive(Debug, Default, Actor, HealthResponder)]
#[services(Actor, MessageSubscriber)]
struct CalculatorActor {}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
struct ServiceStats {
    #[serde(rename = "type")]
    pub typ: String,
    pub name: String,
    pub id: String,
    pub version: String,
    pub metadata: HashMap<String, String>,
    pub endpoints: Vec<EndpointStats>,
    pub started: String,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
struct EndpointStats {
    pub name: String,
    pub subject: String,
    pub queue_group: String,
    pub num_requests: u32,
    pub num_errors: u32,
    pub last_error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
    pub processing_time: u64,
    pub average_processing_time: u64,
}

const SERVICE_NAME: &str = "calculator";
const SERVICE_ID: &str = "adfa8cb7-0821-4e63-9dbd-92a27c30f083";

#[async_trait]
impl MessageSubscriber for CalculatorActor {
    /// handle subscription response
    async fn handle_message(&self, ctx: &Context, msg: &SubMessage) -> RpcResult<()> {
        if msg.subject.starts_with("$SRV") {
            dispatch_micro(ctx, msg.clone()).await?;
        } else {
            dispatch_service(ctx, msg.clone()).await?;
        }

        Ok(())
    }
}

async fn dispatch_micro(ctx: &Context, msg: SubMessage) -> RpcResult<()> {
    let tokens: Vec<String> = msg.subject.split(".").map(|s| s.to_string()).collect();

    match tokens[1].as_str() {
        "PING" => do_ping(ctx, msg).await,
        "STATS" => do_stats(ctx, msg).await,
        "INFO" => do_info(ctx, msg).await,
        _ => {
            // no op
        }
    }
    Ok(())
}

// Reply to ping if:
// $SRV.PING
// $SRV.PING.calculator
// $SRV.PING.calculator.{guid}
async fn do_ping(ctx: &Context, msg: SubMessage) {
    if msg.subject.starts_with("$SRV.PING")
        || msg.subject.starts_with("$SRV.PING.calculator")
        || msg.subject == "$SRV.PING.calculator.adfa8cb7-0821-4e63-9dbd-92a27c30f083"
    {
        ping_respond(ctx, msg.reply_to).await;
    }
}
/*
{
    type: string,
    name: string,
    id: string,
    version: string,
    metadata: Record<string,string>;
} */
async fn ping_respond(ctx: &Context, reply_to: Option<String>) {
    let ping_data = json!({
        "type": "io.nats.micro.v1.ping_response",
        "name": SERVICE_NAME,
        "id": SERVICE_ID,
        "version": "0.1.0",
        "metadata": {}
    });
    let sender = MessagingSender::new();
    sender
        .publish(
            ctx,
            &PubMessage {
                subject: reply_to.unwrap(),
                reply_to: None,
                body: serde_json::to_vec(&ping_data).unwrap(),
            },
        )
        .await
        .unwrap();
}

async fn do_stats(ctx: &Context, msg: SubMessage) {
    // if this was for realsies, we'd get stats from redis
    if msg.subject.ends_with(SERVICE_ID) || msg.subject.ends_with("STATS") || msg.subject.ends_with(SERVICE_NAME) {
        let mut stats = ServiceStats::default();
        stats.typ = "io.nats.micro.v1.stats_response".to_string();
        stats.name = SERVICE_NAME.to_string();
        stats.id = SERVICE_ID.to_string();
        stats.version = "0.1.0".to_string();
        stats.started = "2023-09-18T19:01:45.47046416Z".to_string();
    
        for ep in ["add", "subtract", "multiply"] {
            stats.endpoints.push(EndpointStats {
                name: ep.to_string(),
                subject: ep.to_string(),
                queue_group: "q".to_string(),
                num_requests: 32_312,
                num_errors: 51,
                last_error: Some("this data be fake yarrr".to_string()),
                data: None,
                processing_time: 150,
                average_processing_time: 112,
            });
        }
        let sender = MessagingSender::new();
        sender
            .publish(
                ctx,
                &PubMessage {
                    subject: msg.reply_to.unwrap(),
                    reply_to: None,
                    body: serde_json::to_vec(&stats).unwrap(),
                },
            )
            .await
            .unwrap();
    }
    
}

async fn do_info(ctx: &Context, msg: SubMessage) {
    let info = json!({
        "type": "io.nats.micro.v1.info_response",
        "id": SERVICE_ID,
        "name": SERVICE_NAME,
        "version": "0.1.0",
        "metadata": {},
        "description": "WebAssembly Calculator Service",
        "endpoints": [
            {
                "name": "add",
                "subject": "add",
                "queue_group" : "q",
                "metadata": {}
            },
            {
                "name": "subtract",
                "subject": "subtract",
                "queue_group": "q",
                "metadata": {}
            },
            {
                "name": "multiply",
                "subject": "multiply",
                "queue_group": "q",
                "metadata": {}
            }
        ]
    });
    let sender = MessagingSender::new();
    sender
        .publish(
            ctx,
            &PubMessage {
                subject: msg.reply_to.unwrap(),
                reply_to: None,
                body: serde_json::to_vec(&info).unwrap(),
            },
        )
        .await
        .unwrap();
}

async fn dispatch_service(ctx: &Context, msg: SubMessage) -> RpcResult<()> {
    todo!()
}

/*
if let Some(reply_to) = &msg.reply_to {
            let response = format!("reply: {}", &String::from_utf8_lossy(&msg.body));
            let provider = MessagingSender::new();
            if let Err(e) = provider
                .publish(
                    ctx,
                    &PubMessage {
                        body: response.as_bytes().to_vec(),
                        reply_to: None,
                        subject: reply_to.to_owned(),
                    },
                )
                .await
            {
                error!("sending reply: {}", e.to_string());
            }
        } */
