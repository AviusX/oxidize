use crate::{Context, Error};

use log::error;
use serde::{Serialize, Deserialize};

const PISTON_API: &str = "https://emkc.org/api/v2/piston/execute";

/// Run code
///
/// **Usage:**
/// &code <language_name>
/// \`\`\`
/// <code here>
/// \`\`\`
///
/// **Example:**
/// &code python
/// \`\`\`
/// print("Hello world!")
/// \`\`\`
/// *or*
/// &code
/// \`\`\`py
/// print("Hello world!")
/// \`\`\`
#[poise::command(prefix_command, broadcast_typing, track_edits)]
pub async fn code(
    ctx: Context<'_>,
    // #[description = "The programming language to run"] language: Option<Languages>,
    #[description = "The programming language to run"] language: Option<String>,
    #[description = "The code you want to run"] code: poise::CodeBlock,
) -> Result<(), Error> {
    let runtimes = ctx.data().runtimes.read().await;

    let client = reqwest::Client::new();

    let lang;
    let mut version_number: Option<String> = None;

    if let Some(language) = language {
        lang = language
    } else {
        // get the language from the codeblock
        if let Some(language) = code.language {
            lang = language;
        } else {
            // if even the codeblock doesn't have a language, send an error message
            poise::say_reply(
                ctx,
                "No language provided. Please run `/help code` for command help.",
            ).await?;
            return Ok(());
        }
    }

    for runtime in runtimes.iter() {
        if lang == runtime.language || runtime.aliases.contains(&lang) {
            version_number = Some(runtime.version.clone());
            break;
        }
    }

    if version_number.is_none() {
        poise::say_reply(ctx, format!("Language {} not supported.", lang)).await?;
        return Ok(());
    }

    // construct the code request
    let code_to_run = Code {
        language: lang,
        version: version_number.unwrap(),
        files: [File { content: code.code }],
    };

    // run code using piston
    let response = client
        .post(PISTON_API)
        .json(&code_to_run)
        .send()
        .await?
        .json::<Response>()
        .await?;

    if let Some(err) = response.message {
        // if the request to piston resulted in an error, internally log error and send a message to user.
        error!("Run code failed. Piston error is: \n{}\nCode request was: \n{:#?}", err, code_to_run);
        poise::say_reply(ctx, "There was an error.").await?;
        return Ok(());
    } else {
        // otherwise, send either the code error or output to the user.
        let stderr = &response.run.as_ref().unwrap().stderr;
        let output = &response.run.as_ref().unwrap().output;

        if stderr != "" {
            let error_content = format!(
                "Your code resulted in an error:\n```{}```",
                stderr
            );
            poise::say_reply(ctx, error_content).await?;
        } else {
            let content = format!(
                "Your output is:\n```{}```",
                output
            );
            poise::say_reply(ctx, content).await?;
        }
    }

    Ok(())
}

#[derive(Serialize, Debug)]
struct Code {
    language: String,
    version: String,
    files: [File; 1],
}

#[derive(Serialize, Debug)]
struct File {
    content: String,
}

#[derive(Deserialize, Debug)]
struct Response {
    run: Option<RunResult>,
    message: Option<String>,
}

#[derive(Deserialize, Debug)]
struct RunResult {
    stderr: String,
    output: String,
}
