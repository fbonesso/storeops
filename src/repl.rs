use clap::Parser;
use colored::Colorize;
use rustyline::error::ReadlineError;
use rustyline::history::DefaultHistory;
use rustyline::Editor;

const BANNER: &str = r#"
  ███████╗████████╗ ██████╗ ██████╗ ███████╗ ██████╗ ██████╗ ███████╗
  ██╔════╝╚══██╔══╝██╔═══██╗██╔══██╗██╔════╝██╔═══██╗██╔══██╗██╔════╝
  ███████╗   ██║   ██║   ██║██████╔╝█████╗  ██║   ██║██████╔╝███████╗
  ╚════██║   ██║   ██║   ██║██╔══██╗██╔══╝  ██║   ██║██╔═══╝ ╚════██║
  ███████║   ██║   ╚██████╔╝██║  ██║███████╗╚██████╔╝██║     ███████║
  ╚══════╝   ╚═╝    ╚═════╝ ╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═╝     ╚══════╝
"#;

fn print_banner() {
    println!("{}", BANNER.bright_cyan());
    println!(
        "  {} {}",
        "StoreOps".bold().bright_white(),
        format!("v{}", env!("CARGO_PKG_VERSION")).dimmed()
    );
    println!(
        "  {}",
        "App Store Connect & Google Play — from your terminal.".dimmed()
    );
    println!();
    println!(
        "  Type {} for available commands, {} to leave.",
        "help".bold().bright_yellow(),
        "exit".bold().bright_yellow()
    );
    println!(
        "  {}",
        "─".repeat(60).dimmed()
    );
    println!();
}

fn build_prompt() -> String {
    let profile = match crate::config::Config::load() {
        Ok(cfg) => cfg.active_profile.unwrap_or_default(),
        Err(_) => String::new(),
    };

    if profile.is_empty() {
        format!(
            "{} {} ",
            "storeops".bold().bright_cyan(),
            "›".bright_cyan()
        )
    } else {
        format!(
            "{} {} {} ",
            "storeops".bold().bright_cyan(),
            format!("({})", profile).dimmed(),
            "›".bright_cyan()
        )
    }
}

pub async fn run_repl() {
    print!("\x1B[2J\x1B[1;1H");
    print_banner();

    let mut rl: Editor<(), DefaultHistory> = match Editor::new() {
        Ok(editor) => editor,
        Err(e) => {
            eprintln!("Failed to initialize console: {e}");
            std::process::exit(1);
        }
    };

    let history_path = crate::config::Config::config_dir().map(|d| d.join("history"));
    if let Some(ref path) = history_path {
        let _ = rl.load_history(path);
    }

    loop {
        let prompt = build_prompt();
        match rl.readline(&prompt) {
            Ok(line) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }

                let _ = rl.add_history_entry(line);

                match line {
                    "exit" | "quit" => break,
                    "clear" => {
                        print!("\x1B[2J\x1B[1;1H");
                        print_banner();
                        continue;
                    }
                    _ => {}
                }

                let args = match shell_words::split(line) {
                    Ok(a) => a,
                    Err(e) => {
                        eprintln!("{} {e}", "error:".bright_red().bold());
                        continue;
                    }
                };

                let full_args: Vec<String> = std::iter::once("storeops".to_string())
                    .chain(args)
                    .collect();

                match crate::cli::Cli::try_parse_from(&full_args) {
                    Ok(cli) => {
                        let pretty = cli.pretty;
                        match crate::run(cli).await {
                            Ok(value) => {
                                if pretty {
                                    println!("{}", serde_json::to_string_pretty(&value).unwrap());
                                } else {
                                    println!("{}", serde_json::to_string(&value).unwrap());
                                }
                            }
                            Err(e) => {
                                eprintln!(
                                    "{} {}",
                                    "error:".bright_red().bold(),
                                    e.to_string().bright_red()
                                );
                            }
                        }
                    }
                    Err(e) => {
                        e.print().ok();
                    }
                }
            }
            Err(ReadlineError::Interrupted | ReadlineError::Eof) => break,
            Err(e) => {
                eprintln!("Error: {e}");
                break;
            }
        }
    }

    if let Some(ref path) = history_path {
        let _ = std::fs::create_dir_all(path.parent().unwrap());
        let _ = rl.save_history(path);
    }

    println!("\n  {} 👋\n", "Goodbye!".dimmed());
}
