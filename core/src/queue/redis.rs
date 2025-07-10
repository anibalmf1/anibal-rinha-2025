use redis::{Client, AsyncCommands};
use tracing::{info, error, warn};
use serde::{Serialize, Deserialize};
use crate::config::Settings;
use crate::queue::{QueueConsumerHandler};

#[derive(Clone, Debug, Serialize, Deserialize)]
struct MessageWrapper {
    message: String,
    retry_count: u8,
}

#[derive(Clone, Debug)]
pub struct Producer {
    client: Client,
    queue_name: String,
}

impl Producer {
    pub async fn new(settings: Settings) -> Self {
        let redis_url = settings.redis_url;

        match Client::open(redis_url.clone()) {
            Ok(client) => {
                info!("Connected to Redis at {}", redis_url);
                Self {
                    client,
                    queue_name: settings.payment_topic,
                }
            },
            Err(e) => {
                error!("Failed to connect to Redis at {}: {}", redis_url, e);
                panic!("Failed to connect to Redis");
            }
        }
    }

    pub async fn publish(&self, message: String) -> Result<(), String> {
        info!("Publishing message to queue: {}", self.queue_name);

        let mut conn = match self.client.get_async_connection().await {
            Ok(conn) => conn,
            Err(e) => {
                error!("Failed to get Redis connection: {}", e);
                return Err(format!("Failed to get Redis connection: {}", e));
            }
        };

        // Wrap the message with retry information
        let wrapped_message = MessageWrapper {
            message,
            retry_count: 0,
        };

        // Serialize the wrapped message
        let serialized = match serde_json::to_string(&wrapped_message) {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to serialize message: {}", e);
                return Err(format!("Failed to serialize message: {}", e));
            }
        };

        // Use RPUSH to add the message to the end of the list
        match conn.rpush::<_, _, ()>(&self.queue_name, serialized).await {
            Ok(_) => {
                info!("Message published to Redis successfully");
                Ok(())
            },
            Err(e) => {
                error!("Failed to publish message to Redis: {}", e);
                Err(format!("Failed to publish message to Redis: {}", e))
            }
        }
    }
}

pub struct Consumer {
    client: Client,
    queue_name: String,
    dlq_name: String,
}

impl Consumer {
    pub async fn new(settings: Settings) -> Self {
        let redis_url = settings.redis_url.clone();
        let topic = settings.payment_topic.clone();

        match Client::open(redis_url.clone()) {
            Ok(client) => {
                info!("Connected to Redis at {}", redis_url);
                Self {
                    client,
                    queue_name: topic.to_string(),
                    dlq_name: format!("{}_dlq", topic),
                }
            },
            Err(e) => {
                error!("Failed to connect to Redis at {}: {}", redis_url, e);
                panic!("Failed to connect to Redis");
            }
        }
    }

    pub async fn start_consuming(&self, handler: impl QueueConsumerHandler) {
        info!("Starting to consume messages from queue: {}", self.queue_name);
        info!("DLQ configured as: {}", self.dlq_name);

        let client = self.client.clone();
        let queue_name = self.queue_name.clone();
        let dlq_name = self.dlq_name.clone();

        tokio::spawn(async move {
            loop {
                let mut conn = match client.get_async_connection().await {
                    Ok(conn) => conn,
                    Err(e) => {
                        error!("Failed to get Redis connection: {}", e);
                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                        continue;
                    }
                };

                // Use BLPOP to get the first message from the list
                let result: redis::RedisResult<Option<(String, String)>> = conn.blpop(&queue_name, 0f64).await;

                match result {
                    Ok(Some((_, serialized_wrapper))) => {
                        // Deserialize the wrapped message
                        let wrapper = match serde_json::from_str::<MessageWrapper>(&serialized_wrapper) {
                            Ok(w) => w,
                            Err(e) => {
                                error!("Failed to deserialize message wrapper: {}", e);
                                // If we can't deserialize, we can't retry properly, so just continue
                                continue;
                            }
                        };

                        info!("Received message from Redis (retry count: {})", wrapper.retry_count);

                        // Process the original message
                        match handler.consume(wrapper.message.clone()).await {
                            Ok(_) => {
                                info!("Message processed successfully");
                            },
                            Err(e) => {
                                // Increment retry count
                                let new_retry_count = wrapper.retry_count + 1;
                                error!("Error processing message: {}, retry count: {}", e, new_retry_count);

                                // Create a new wrapper with incremented retry count
                                let new_wrapper = MessageWrapper {
                                    message: wrapper.message,
                                    retry_count: new_retry_count,
                                };

                                // Serialize the new wrapper
                                let serialized = match serde_json::to_string(&new_wrapper) {
                                    Ok(s) => s,
                                    Err(ser_err) => {
                                        error!("Failed to serialize message wrapper: {}", ser_err);
                                        continue;
                                    }
                                };

                                // If retry count is less than 3, push back to the original queue
                                // Otherwise, push to the DLQ
                                let target_queue = if new_retry_count < 3 {
                                    &queue_name
                                } else {
                                    info!("Message retry limit reached, moving to DLQ: {}", dlq_name);
                                    &dlq_name
                                };

                                // Push the message to the appropriate queue
                                if let Err(push_err) = conn.rpush::<_, _, ()>(target_queue, serialized).await {
                                    error!("Failed to push message to queue {}: {}", target_queue, push_err);
                                }
                            }
                        }
                    },
                    Ok(None) => {
                        warn!("BLPOP didn't return any messages")
                    },
                    Err(e) => {
                        error!("Error receiving message from Redis: {}", e);
                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                    }
                }
            }
        });
    }
}
