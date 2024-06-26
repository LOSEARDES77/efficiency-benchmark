use std::env::args;
use std::fs::{create_dir, read_dir, remove_dir_all};
use std::io::{stdin, stdout, Write};
use std::process::exit;
use std::thread::sleep;
use std::time::Duration;
use colored::Colorize;
use efficiency_benchmark::{get_battery_percentage, bench};


fn main() {

    let mut repo_url = "https://github.com/rust-lang/rustlings.git";
    let mut build_command = String::from("cargo build");

    let args = args().collect::<Vec<String>>();
    let mut has_arg_repo = false;
    let mut has_arg_build = false;
    for i in 0..args.len() {
        if args[i] == "--help" || args[i] == "-h" {
            #[cfg(unix)]
            println!("Usage: {} [OPTIONS]", args[0].split("/").collect::<Vec<&str>>().last().unwrap());
            #[cfg(windows)]
            println!("Usage: {} [OPTIONS]", args[0].split("\\").collect::<Vec<&str>>().last().unwrap());
            println!();
            println!("Options:");
            println!("  --help, -h    Show this help message");
            println!("  --repo, -r    Set the repository URL to clone");
            println!("  --build, -b   Set the build command");
            println!();
            println!("If no args provided it will use by default:");
            println!("  repo-url   -> https://github.com/rust-lang/rustlings.git");
            println!("  build-cmd  -> cargo build");
            println!();
            println!("This benchmark is meant for laptops,");
            println!("It will infinitely loop compiling something until it runs out of battery");
            println!("That's how this benchmark works");
            println!();
            exit(0);
        }
        if args[i] == "--repo" || args[i] == "-r" {
            has_arg_repo = true;
            if i + 1 < args.len() {
                repo_url = &args[i + 1];
            }
        }
        if args[i] == "--build" || args[i] == "-b" {
            has_arg_build = true;
            if i + 1 < args.len() {
                let mut new_build_command = String::new();
                for j in i + 1..args.len() {
                    if args[j].contains("--") || args[j].contains("-"){
                        break;
                    }
                    new_build_command = format!("{} {}", new_build_command, args[j].as_str());
                }
                build_command = new_build_command;
            }
        }
    }

    let repo_url = repo_url.trim();
    let build_command = build_command.trim();

    if has_arg_repo {
        let is_valid_repo_url = repo_url.starts_with("http://") || repo_url.starts_with("https://") || repo_url.starts_with("git@");
        println!("{}", is_valid_repo_url);
        if !is_valid_repo_url {
            println!("[{}] Invalid repository URL", "ERROR".red());
            exit(1);
        }
        println!("Using repository: {}", repo_url);
    }

    if has_arg_build {
        println!("Using build command: {}", build_command);
    }

    let battery_percentage = get_battery_percentage();
    if battery_percentage < 100 {
        println!("[{}] Battery is not full, you might get a lower score", "WARNING".red());
        print!("Would you like to continue? [Y/N] ");
        stdout().flush().unwrap();
        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();
        if input.trim().to_lowercase() == "y" {
            print!("Would you like to wait until battery is full? [Y/N] ");
            stdout().flush().unwrap();
            let mut input = String::new();
            stdin().read_line(&mut input).unwrap();
            if input.trim().to_lowercase() == "y" {
                loop {
                    if battery_percentage == 100 {
                        break;
                    }
                    sleep(Duration::from_secs(1));
                }
            }
        }else {
            println!("Exiting...");
            exit(1);
        }
    }

    #[cfg(unix)]
    let app_dir = format!("{}/.local/share/{}", std::env::var("HOME").expect("No HOME directory"), std::env::var("CARGO_PKG_NAME").expect("No CARGO_PKG_NAME"));
    #[cfg(windows)]
    let app_dir = format!("{}\\{}", std::env::var("APP_DATA").expect("No APP_DATA directory"), std::env::var("CARGO_PKG_NAME").expect("No CARGO_PKG_NAME"));

    let app_dir = app_dir.as_str();

    if  !read_dir(app_dir).is_ok() {
        create_dir(app_dir).unwrap();
    }

    let source_dir = format!("{}/repo-dir", app_dir);
    let build_dir = format!("{}/build-dir", app_dir);

    let source_dir = source_dir.as_str();
    let build_dir = build_dir.as_str();



    let repo_dir = read_dir(source_dir);
    let mut has_asked = false;
    let mut repo_exists = false;
    if repo_dir.is_ok() {
        for entry in repo_dir.unwrap() {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_dir() && !has_asked {
                has_asked = true;
                repo_exists = true;
                print!("Repo directory already exists, would you like to delete it? [Y/N] ");
                stdout().flush().unwrap();
                let mut input = String::new();
                stdin().read_line(&mut input).unwrap();
                if input.trim().to_lowercase() == "y" {
                    remove_dir_all(source_dir).unwrap();
                    repo_exists = false;
                }
            }
        }
    }
    


    let iterator = bench(repo_url, build_command, source_dir, build_dir, repo_exists);

    for line in iterator {
        println!("{}", line);
    }
    

}


