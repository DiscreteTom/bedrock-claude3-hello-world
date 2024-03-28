use aws_sdk_bedrockruntime as bedrockruntime;
use bedrockruntime::primitives::Blob;
use serde_json::json;
use tokio::io::{self, AsyncReadExt};

#[::tokio::main]
async fn main() -> Result<(), bedrockruntime::Error> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_bedrockruntime::Client::new(&config);

    loop {
        let mut buf = vec![0; 1024];
        let mut stdin = io::stdin();
        let n = match stdin.read(&mut buf).await {
            Err(_) | Ok(0) => break,
            Ok(n) => n,
        };
        let line = String::from_utf8(buf).unwrap();

        let mut res = client
            .invoke_model_with_response_stream()
            .content_type("application/json")
            .model_id("anthropic.claude-3-haiku-20240307-v1:0")
            .body(Blob::new(
                json! {
                    {
                        "anthropic_version": "bedrock-2023-05-31",
                        "max_tokens": 1000,
                        "messages": [
                          {
                            "role": "user",
                            "content": [{ "type": "text", "text": line }],
                          },
                        ],
                      }
                }
                .to_string()
                .into_bytes(),
            ))
            .send()
            .await?;

        loop {
            match res.body.recv().await {
                Ok(option_stream) => match option_stream {
                    Some(stream) => {
                        if let Ok(chunk) = stream.as_chunk() {
                            println!(
                                "{}",
                                String::from_utf8(chunk.bytes.clone().unwrap().into_inner())
                                    .unwrap()
                            );
                        } else {
                            break;
                        };
                    }
                    None => {
                        eprintln!("No stream");
                        break;
                    }
                },
                Err(e) => {
                    eprintln!("{:?}", e);
                    break;
                }
            }
        }
    }

    Ok(())
}
