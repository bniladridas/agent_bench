use std::env;
use dotenv::dotenv;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde_json::json;
use rusqlite::{params, Connection};
use uuid::Uuid;
use std::io::{self, Write};
use colored::*;
use std::fs::File;
use std::time::Duration;
use chrono::Datelike;

#[derive(Debug, Clone, Copy)]
enum ApiProvider {
    OpenAI,
    Sambanova,
    Gemini,
}

#[derive(Debug)]
struct ApiConfig {
    provider: ApiProvider,
    api_key: String,
    base_url: String,
    model_name: String,
}

#[derive(Debug)]
struct Message {
    role: String,
    content: String,
}

fn init_db(conn: &Connection) {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS sessions (
            id TEXT PRIMARY KEY,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    ).unwrap();
    conn.execute(
        "CREATE TABLE IF NOT EXISTS messages (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            session_id TEXT,
            role TEXT,
            content TEXT,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(session_id) REFERENCES sessions(id)
        )",
        [],
    ).unwrap();
}

fn save_message(conn: &Connection, session_id: &str, role: &str, content: &str) {
    conn.execute(
        "INSERT INTO messages (session_id, role, content) VALUES (?1, ?2, ?3)",
        params![session_id, role, content],
    ).unwrap();
}

fn save_session(conn: &Connection, session_id: &str) {
    conn.execute(
        "INSERT OR IGNORE INTO sessions (id) VALUES (?1)",
        params![session_id],
    ).unwrap();
}

fn list_sessions(conn: &Connection) {
    let mut stmt = conn.prepare("SELECT id, created_at FROM sessions ORDER BY created_at DESC").unwrap();
    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    }).unwrap();
    println!("{}", "Previous Sessions:".bold().yellow());
    for (i, row) in rows.enumerate() {
        let (id, created_at) = row.unwrap();
        println!("{}: {} ({})", i + 1, id, created_at);
    }
}

fn load_history(conn: &Connection, session_id: &str) -> Vec<Message> {
    let mut stmt = conn.prepare("SELECT role, content FROM messages WHERE session_id = ?1 ORDER BY id ASC").unwrap();
    let rows = stmt
        .query_map(params![session_id], |row| {
            Ok(Message {
                role: row.get(0)?,
                content: row.get(1)?,
            })
        })
        .unwrap();
    rows.map(|m| m.unwrap()).collect()
}

fn view_session(conn: &Connection) {
    print!("Enter session ID to view: ");
    io::stdout().flush().unwrap();
    let mut session_id = String::new();
    io::stdin().read_line(&mut session_id).unwrap();
    let session_id = session_id.trim();
    let history = load_history(conn, session_id);
    println!("\n{}\n", "Session History:".bold().yellow());
    for msg in history {
        match msg.role.as_str() {
            "user" => println!("{} {}", "You:".bold().blue(), msg.content.blue()),
            "assistant" => println!("{} {}", "Assistant:".bold().green(), msg.content.green()),
            "system" => println!("{} {}", "System:".bold().magenta(), msg.content.magenta()),
            _ => println!("{}: {}", msg.role, msg.content),
        }
    }
}

fn export_session(conn: &Connection) {
    print!("Enter session ID to export: ");
    io::stdout().flush().unwrap();
    let mut session_id = String::new();
    io::stdin().read_line(&mut session_id).unwrap();
    let session_id = session_id.trim();
    let history = load_history(conn, session_id);
    let filename = format!("session_{}.txt", session_id);
    let mut file = File::create(&filename).unwrap();
    for msg in &history {
        let line = format!("{}: {}\n", msg.role, msg.content);
        file.write_all(line.as_bytes()).unwrap();
    }
    println!("Session exported to {}", filename.bold().yellow());
}

async fn web_search(query: &str) -> Result<String, reqwest::Error> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()?;
    let url = format!("https://api.duckduckgo.com/?q={}&format=json", query);
    let response = client.get(&url).send().await?;

    if !response.status().is_success() {
        let error_text = format!("Search API returned a non-success status: {}. Body: {}", response.status(), response.text().await.unwrap_or_else(|_| "Could not read body".to_string()));
        return Ok(error_text);
    }
    
    response.text().await
}

async fn call_llm(client: &reqwest::Client, config: &ApiConfig, history: &[Message]) -> Result<String, Box<dyn std::error::Error>> {
    let res = match config.provider {
        ApiProvider::OpenAI | ApiProvider::Sambanova => {
            let messages_json: Vec<_> = history.iter().map(|m| json!({"role": m.role, "content": m.content})).collect();
            let body = json!({
                "model": config.model_name,
                "messages": messages_json,
                "temperature": 0.1,
                "top_p": 0.1
            });
            client
                .post(&config.base_url)
                .header(AUTHORIZATION, format!("Bearer {}", config.api_key))
                .header(CONTENT_TYPE, "application/json")
                .json(&body)
                .send()
                .await?
        }
        ApiProvider::Gemini => {
            // Gemini uses 'model' for assistant and 'user' for user.
            // It also expects contents to not have adjacent same roles.
            let mut gemini_contents = Vec::new();
            if let Some(first_message) = history.first() {
                 if first_message.role == "system" {
                    gemini_contents.push(json!({
                        "role": "user",
                        "parts": [{"text": first_message.content}]
                    }));
                    gemini_contents.push(json!({
                        "role": "model",
                        "parts": [{"text": "Understood."}]
                    }));
                }
            }

            for msg in history.iter().skip(1) {
                let role = if msg.role == "assistant" { "model" } else { "user" };
                gemini_contents.push(json!({
                    "role": role,
                    "parts": [{"text": msg.content}]
                }));
            }

            let body = json!({
                "contents": gemini_contents
            });
            let url = format!("{}?key={}", config.base_url, config.api_key);
            client
                .post(&url)
                .header(CONTENT_TYPE, "application/json")
                .json(&body)
                .send()
                .await?
        }
    };

    if !res.status().is_success() {
        let status = res.status();
        let error_text = res.text().await.unwrap_or_else(|_| "Could not read error body".to_string());
        return Err(format!("API Error: {} ({})", error_text, status).into());
    }

    let resp_json: serde_json::Value = res.json().await.unwrap_or_else(|_| json!({}));

    let assistant_reply = match config.provider {
        ApiProvider::OpenAI | ApiProvider::Sambanova => {
            resp_json["choices"][0]["message"]["content"].as_str().unwrap_or("[No response]").to_string()
        }
        ApiProvider::Gemini => {
            resp_json["candidates"][0]["content"]["parts"][0]["text"].as_str().unwrap_or("[No response]").to_string()
        }
    };

    Ok(assistant_reply)
}

async fn start_chat_session(conn: &Connection, config: &ApiConfig) {
    let session_id = Uuid::new_v4().to_string();
    save_session(&conn, &session_id);

    print!("Enable web search for this session? (y/n): ");
    io::stdout().flush().unwrap();
    let mut web_search_choice = String::new();
    io::stdin().read_line(&mut web_search_choice).unwrap();
    let web_search_enabled = web_search_choice.trim().eq_ignore_ascii_case("y");

    println!("{}\n", "New chat session started. Type 'exit' to quit.".bold().yellow());
    
    let system_prompt = if web_search_enabled {
        let current_year = chrono::Local::now().year();
        format!(
            "You are a helpful AI assistant powered by the {} model.
You have the ability to run any Linux shell command.
Your response MUST be ONLY the tool command. Do not add any explanation.
Do NOT use interactive commands (like 'nano', 'vim'). Use non-interactive commands like `cat` to read files.

Tool format:
- Run a shell command: `[RUN_COMMAND <command to run>]`
- Search the web: `[SEARCH: your query]`. Current year: {}",
            config.model_name, current_year
        )
    } else {
        format!("You are an AI assistant powered by the {} model.", config.model_name)
    };

    let mut history = vec![
        Message { role: "system".to_string(), content: system_prompt }
    ];

    loop {
        print!("{} ", "You:".bold().blue());
        io::stdout().flush().unwrap();
        let mut user_input = String::new();
        io::stdin().read_line(&mut user_input).unwrap();
        let user_input = user_input.trim();

        if user_input.is_empty() {
            continue;
        }
        
        if user_input.eq_ignore_ascii_case("exit") || user_input.eq_ignore_ascii_case("quit") {
            println!("{}", "Session ended.".bold().yellow());
            break;
        }

        history.push(Message { role: "user".to_string(), content: user_input.to_string() });
        save_message(&conn, &session_id, "user", user_input);

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(90))
            .build()
            .unwrap();

        match call_llm(&client, config, &history).await {
            Ok(mut assistant_reply) => {
                let trimmed_reply = assistant_reply.trim().trim_matches(|c| c == '\'' || c == '\"' || c == '`');

                let mut tool_used = false;

                if trimmed_reply.to_uppercase().starts_with("[RUN_COMMAND") {
                    tool_used = true;
                    let command_str = if let Some(pos) = trimmed_reply.find(' ') {
                        trimmed_reply[pos..].trim_start().trim_end_matches(']')
                    } else {
                        ""
                    };

                    if command_str.is_empty() {
                        println!("{} {}", "System:".bold().magenta(), "No command provided for [RUN_COMMAND].".red());
                        continue;
                    }

                    println!("{} Running command: {}", "System:".bold().magenta(), command_str.magenta());

                    let output = std::process::Command::new("sh")
                        .arg("-c")
                        .arg(command_str)
                        .output()
                        .expect("failed to execute process");

                    let result = if output.status.success() {
                        String::from_utf8_lossy(&output.stdout).to_string()
                    } else {
                        String::from_utf8_lossy(&output.stderr).to_string()
                    };
                    
                    println!("{}\n{}", "Assistant:".bold().green(), result.green());
                    history.push(Message { role: "assistant".to_string(), content: assistant_reply.clone() });
                    history.push(Message { role: "system".to_string(), content: format!("Command output:\n{}", result) });
                } else if web_search_enabled && trimmed_reply.to_uppercase().starts_with("[SEARCH:") {
                    tool_used = true;
                    let query_part = trimmed_reply.splitn(2, ':').nth(1).unwrap_or("").trim_end_matches(']');
                    println!("{} Searching the web for: {}", "System:".bold().magenta(), query_part.magenta());
                    
                    let search_results = web_search(query_part).await.unwrap_or_else(|e| format!("Failed to perform web search: {}", e));
                    let tool_result_prompt = format!("Web search results for '{}':\n{}", query_part, search_results);
                    history.push(Message { role: "assistant".to_string(), content: assistant_reply.clone() });
                    history.push(Message { role: "system".to_string(), content: tool_result_prompt });
                }

                if tool_used {
                    match call_llm(&client, config, &history).await {
                        Ok(final_reply) => {
                            assistant_reply = final_reply;
                        }
                        Err(e) => {
                             println!("Assistant: {} ({})", "API Error after tool use".red(), e.to_string().red());
                            continue;
                        }
                    }
                }

                println!("{} {}\n", "Assistant:".bold().green(), assistant_reply.green());
                history.push(Message { role: "assistant".to_string(), content: assistant_reply.clone() });
                save_message(&conn, &session_id, "assistant", &assistant_reply);
            },
            Err(e) => {
                println!("Assistant: {} ({})", "API Error".red(), e.to_string().red());
                continue;
            }
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    println!("{}", "Select an API Provider:".bold().yellow());
    println!("1. OpenAI (gpt-4-turbo)");
    println!("2. Sambanova (Meta-Llama-3.2-1B-Instruct)");
    println!("3. Google Gemini (gemini-2.0-flash)");
    print!("Enter your choice: ");
    io::stdout().flush().unwrap();

    let mut choice = String::new();
    io::stdin().read_line(&mut choice).unwrap();

    let config = match choice.trim() {
        "1" => ApiConfig {
            provider: ApiProvider::OpenAI,
            api_key: env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set in .env for OpenAI"),
            base_url: "https://api.openai.com/v1/chat/completions".to_string(),
            model_name: "gpt-4-turbo".to_string(),
        },
        "2" => ApiConfig {
            provider: ApiProvider::Sambanova,
            api_key: env::var("SAMBANOVA_API_KEY").expect("SAMBANOVA_API_KEY not set in .env for Sambanova"),
            base_url: "https://api.sambanova.ai/v1/chat/completions".to_string(),
            model_name: "Meta-Llama-3.2-1B-Instruct".to_string(),
        },
        "3" => ApiConfig {
            provider: ApiProvider::Gemini,
            api_key: env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY not set in .env for Google Gemini"),
            base_url: "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash:generateContent".to_string(),
            model_name: "gemini-2.0-flash".to_string(),
        },
        _ => {
            println!("{}", "Invalid choice. Exiting.".red());
            return;
        }
    };

    let conn = Connection::open("chat_sessions.db").unwrap();
    init_db(&conn);

    loop {
        println!("\n{}", "Main Menu".bold().yellow());
        println!("1. Start new chat session");
        println!("2. List previous sessions");
        println!("3. View a session's history");
        println!("4. Export a session's history");
        println!("5. Quit");
        print!("Enter your choice: ");
        io::stdout().flush().unwrap();

        let mut menu_choice = String::new();
        io::stdin().read_line(&mut menu_choice).unwrap();

        match menu_choice.trim() {
            "1" => start_chat_session(&conn, &config).await,
            "2" => list_sessions(&conn),
            "3" => view_session(&conn),
            "4" => export_session(&conn),
            "5" => {
                println!("{}", "Goodbye!".bold().yellow());
                break;
            },
            _ => println!("{}", "Invalid choice. Please try again.".red()),
        }
    }
}
