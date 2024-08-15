use dotenv::dotenv;
use std::env;
use serde::{Deserialize, Serialize};
use ureq;
use std::io::{self, Write};
use std::fs::OpenOptions;
use std::fs::File;
use std::io::prelude::*;

#[derive(Serialize, Deserialize, Debug)]
struct Message {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct RequestPayload {
    messages: Vec<Message>,
    temperature: f32,
    top_p: f32,
    max_tokens: u32,
}

#[derive(Deserialize, Debug)]
struct Choice {
    message: Message,
    finish_reason: String,
    index: u32,
}

#[derive(Deserialize, Debug)]
struct ResponsePayload {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<Choice>,
    usage: Usage,
}

#[derive(Deserialize, Debug)]
struct Usage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

fn get_user_input(prompt: &str) -> String {
    use std::io::{self, Write};

    print!("{}", prompt);
    io::stdout().flush().expect("Failed to flush stdout");

    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");

    input.trim().to_owned()
}


fn generate_prompt(user_question: &str, language: &str, option: &str) -> String {
    let prompt = match option {
        "1" => format!(
            "Look into the user question: {}. Try to help as much as possible to complete the code. The preferred language is {}.",
            user_question, language
        ),
        "2" => format!(
            "Look into the user question: {}. The code is written in {}. Try to help as much as possible by explaining the code in simple words.",
            user_question, language
        ),
        "3" => format!(
            "Look into the user question: {}. Try to help as much as possible by giving code refactoring suggestions. The code is written in {}.",
            user_question, language
        ),
        _ => "Invalid option provided.".to_string(),
    };
    prompt
}


fn language_model(prompt_input: &str, language: &str, option: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!();

    dotenv().ok();

    let api_endpoint = env::var("API_ENDPOINT").map_err(|e| format!("Failed to get API_ENDPOINT: {}", e))?;
    let api_key = env::var("API_KEY").map_err(|e| format!("Failed to get API_KEY: {}", e))?;

    let generated_prompt = generate_prompt(prompt_input, language, option);

    let payload = RequestPayload {
        messages: vec![Message {
            role: "user".to_string(),
            content: generated_prompt.clone(),
        }],
        temperature: 0.7,
        top_p: 0.95,
        max_tokens: 800,
    };

    let response = ureq::post(&api_endpoint)
        .set("Content-Type", "application/json")
        .set("api-key", &api_key)
        .send_json(&payload)?;

    let response_from_server: ResponsePayload = response.into_json()?;
    let text = &response_from_server.choices[0].message.content;

    println!("{}", text);

    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open("chat_history")?;

    writeln!(file, "Prompt: {}\n", generated_prompt)?;
    writeln!(file, "Response: {}\n", text)?;

    Ok(())
}




fn handle_code_task(option: &str) {
    let lang = get_user_input("Please enter the coding language: ");
    println!();

    let input_method = get_user_input("To type your code here (1) to cancel (2). Enter 1 or 2: ");
    println!();

    let prompt_input = match input_method.as_str() {
        "1" => {
            let code = get_user_input("Please enter your code here: ");
            code.trim().to_string()
        },
        "2" => {
            let mut file = File::open("input").unwrap_or_else(|_| {
                println!("You have cenceled request!");
                std::process::exit(1);
            });

            let mut contents = String::new();
            file.read_to_string(&mut contents).expect("Could not read the file");
            contents.trim().to_string()
        },
        _ => {
            println!("Invalid choice. Please try again.");
            return;
        }
    };

    if let Err(err) = language_model(&prompt_input, &lang, option) {
        eprintln!("Error: {}", err);
    }
}


fn display_chat_history() {
    match File::open("chat_history") {
        Ok(mut file) => {
            let mut contents = String::new();
            if file.read_to_string(&mut contents).is_ok() {
                if contents.is_empty() {
                    println!("No chat history found.");
                } else {
                    println!("\nChat History:\n{}", contents);
                }
            } else {
                println!("Could not read the file.");
            }
        }
        Err(_) => println!("No chat history found."),
    }
}

fn clear_chat_history() {
    match File::create("chat_history") {
        Ok(mut file) => {
            if file.set_len(0).is_ok() {
                println!("Chat history is cleared.");
            } else {
                println!("Failed to clear the chat history.");
            }
        }
        Err(_) => println!("Could not clear the chat history."),
    }
}

fn print_help() {
    println!("\nHelp - AI Code Assistant\n");
    println!("(1)Code Completion: Provide your code and get assistance with code completion.");
    println!("(2) Code Explanation: Provide code and receive an explanation.");
    println!("(3) Chat History: View previous prompts and responses.");
    println!("(4) Clear Chat History: Remove all saved prompts and responses.");
    println!("(6) Exit: Close the application.\n");
    println!("For options 1 & 2, type your own code.\n");
}

fn main() {
    loop {
        println!("\nAI Code Assistant:\n");
        println!("(1) Code Completion");
        println!("(2) Code Explanation");
        println!("(3) Chat History");
        println!("(4) Clear Chat History");
        println!("(5) Help");
        println!("(6) Exit\n");
        print!("Please choose an option from above: ");
        io::stdout().flush().unwrap();

        let choice = get_user_input("");
        println!();

        match choice.as_str() {
            "1" | "2" => handle_code_task(&choice),
            "3" => display_chat_history(),
            "4" => clear_chat_history(),
            "5" => print_help(),
            "6" => break,
            _ => println!("Invalid option, please try again."),
        }
    }
}
