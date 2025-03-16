use anyhow::Result;
use mistralrs::{
    GgufModelBuilder, PagedAttentionMetaBuilder, RequestBuilder, TextMessageRole, TextMessages,
};
use serde_json::json;

async fn run() -> Result<()> {
    // We do not use any files from remote servers here, and instead load the
    // chat template from the specified file, and the tokenizer and model from a
    // local GGUF file at the path specified.
    let model = GgufModelBuilder::new("gguf_models/", vec!["unsloth.Q4_K_M.gguf"])
        // .with_chat_template("chat_templates/llama3.json")
        // .with_logging()
        // .with_paged_attn(|| PagedAttentionMetaBuilder::default().build())?
        .build()
        .await?;

    let json_schema = json!({
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "joke": {
            "description": "joke",
            "type": "string",
          }
        }
      }
    });

    let messages = RequestBuilder::new()
        // .set_sampler_temperature(0.5)
        .set_constraint(mistralrs::Constraint::JsonSchema(json_schema))
        .add_message(TextMessageRole::System, "Your are a commedian")
        .add_message(
            TextMessageRole::User,
            "Please write me 2 jokes in a json array",
        );

    let response = model.send_chat_request(messages).await?;

    println!("{}", response.choices[0].message.content.as_ref().unwrap());
    dbg!(
        response.usage.avg_prompt_tok_per_sec,
        response.usage.avg_compl_tok_per_sec
    );

    // Next example: Return some logprobs with the `RequestBuilder`, which enables higher configurability.
    let request = RequestBuilder::new().return_logprobs(true).add_message(
        TextMessageRole::User,
        "Please write a mathematical equation where a few numbers are added.",
    );

    let response = model.send_chat_request(request).await?;

    println!(
        "Logprobs: {:?}",
        &response.choices[0]
            .logprobs
            .as_ref()
            .unwrap()
            .content
            .as_ref()
            .unwrap()[0..3]
    );

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    run().await?;
    run().await?;
    println!("Sleep");
    tokio::time::sleep(tokio::time::Duration::from_secs(25)).await;
    Ok(())
}
