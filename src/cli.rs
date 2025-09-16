use std::env;
use std::path::PathBuf;

#[derive(Debug)]
pub struct Args {
    pub config_file: PathBuf,
    pub dry_run: bool,
    pub working_directory: PathBuf,
}

impl Args {
    pub fn parse() -> Self {
        let args: Vec<String> = env::args().collect();
        let mut config_file = PathBuf::from(".release-config.toml");
        let mut dry_run = false;
        let mut working_directory = PathBuf::from(".");

        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "--config-file" => {
                    if i + 1 < args.len() {
                        config_file = PathBuf::from(&args[i + 1]);
                        i += 2;
                    } else {
                        eprintln!("Error: --config-file requires a value");
                        std::process::exit(1);
                    }
                }
                "--dry-run" => {
                    dry_run = true;
                    i += 1;
                }
                "--working-directory" => {
                    if i + 1 < args.len() {
                        working_directory = PathBuf::from(&args[i + 1]);
                        i += 2;
                    } else {
                        eprintln!("Error: --working-directory requires a value");
                        std::process::exit(1);
                    }
                }
                "--help" | "-h" => {
                    Self::print_help();
                    std::process::exit(0);
                }
                _ => {
                    eprintln!("Error: Unknown argument: {}", args[i]);
                    eprintln!("Use --help for usage information");
                    std::process::exit(1);
                }
            }
        }

        Self {
            config_file,
            dry_run,
            working_directory,
        }
    }

    pub fn from_env() -> Self {
        Self {
            config_file: env::var("CONFIG_FILE")
                .unwrap_or_else(|_| ".release-config.toml".to_string())
                .into(),
            dry_run: env::var("DRY_RUN")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
            working_directory: env::var("WORKING_DIRECTORY")
                .unwrap_or_else(|_| ".".to_string())
                .into(),
        }
    }

    fn print_help() {
        println!("conventional-release-action");
        println!("A flexible, config-driven release flow that scales from a single package to large monorepos");
        println!();
        println!("OPTIONS:");
        println!("    --config-file <FILE>           Path to the configuration file [default: .release-config.toml]");
        println!("    --dry-run                      Run in dry-run mode without creating releases");
        println!("    --working-directory <DIR>      Working directory [default: .]");
        println!("    --help, -h                     Print help information");
    }
}
