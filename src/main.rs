use std::env::{args, set_current_dir};
use std::fs::{copy, create_dir, create_dir_all, metadata, read_dir, remove_dir_all, remove_file, File};
use std::io::{BufRead, BufReader, stdout, Write, stdin};
use std::process::{Command, exit, Stdio};
use std::thread::sleep;
use std::time::Duration;
use battery::{Manager, State};
use colored::Colorize;
use chrono::Local;


fn main() {

    let mut repo_url = "https://github.com/rust-lang/rustlings.git";
    let mut build_command = String::from("cargo build");
    let source_dir = "repo-dir";
    let build_dir = "build-dir";

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

    // check if directory exists
    if  !read_dir("benchmark").is_ok() {
        create_dir("benchmark").unwrap();
    }

    set_current_dir("benchmark").unwrap();

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
    
    if !repo_exists {
        let mut command = Command::new("git")
            .arg("clone")
            .arg("--progress")
            .arg(repo_url)
            .arg("--recursive")
            .arg(source_dir)
            .stderr(Stdio::piped())
            .spawn()
            .expect("failed to clone linux kernel");
    
        let reader = BufReader::new(command.stderr.take().expect("failed to get stderr"));
        for line in reader.lines() {
            match line {
                Ok(line) => {
                    println!("{}", line);
                },
                Err(_) => {},
            }
        }
    
        command.wait().expect("failed to wait for command");
    }

    if metadata(build_dir).is_ok() {
        remove_dir_all(build_dir).unwrap();
    }

    if is_plugged(false) {
        println!("Please unplug the system to start th benchmarking");
        loop {
            if !is_plugged(true){
                break;
            }
            sleep(Duration::from_secs(1));
        }
    }
    
    let current_time = Local::now().format("%d-%m-%Y_%H:%M:%S").to_string();
    let logfile = &format!("benchmark-{}.log", current_time);
    if metadata(logfile).is_ok() {
        remove_file(logfile).unwrap();
    }
    
    loop {
        // Copy build dir
        println!("Copying repo");
        copy_directory(source_dir, build_dir).expect("failed to copy src directory");

        set_current_dir(build_dir).unwrap();
        
        // Build
        println!("Building");
        execute_build_command(&build_command);

        // Delete build dir
        set_current_dir("../").unwrap();
        remove_dir_all(build_dir).unwrap();
        
        // Add score
        println!("Build successful!");
        
        add_one(logfile);
    }
    

}

fn add_one(logfile: &str) {
    
    if !metadata(logfile).is_ok() {
        let mut file = File::create(logfile).unwrap();
        file.write_all("0".as_bytes()).unwrap();
    }

    let mut reader = BufReader::new(File::open(logfile).unwrap());
    let mut score = Vec::new(); // Change the type of score to Vec<u8>
    reader.read_until(b'\n', &mut score).unwrap();
    let score = String::from_utf8_lossy(&score).parse::<u32>().unwrap(); // Parse the score as u32
    let score = score + 1; // Increment the score
    let mut file = File::create(logfile).unwrap();
    file.write_all(score.to_string().as_bytes()).unwrap();
    println!("Current Score: {}", score);
    sleep(Duration::from_secs(1));
}

fn get_battery_percentage() -> u8 {
    let manager = Manager::new().unwrap().batteries().unwrap();
    let battery = match manager.into_iter().next(){
        Some(battery) => battery.unwrap(),
        None => {
            return 100 as u8;
        },
    
    };
    let percentage = battery.state_of_charge().value * 100.0;
    return percentage as u8;
}

fn is_plugged(has_asked: bool) -> bool {
    let manager = Manager::new().unwrap().batteries().unwrap();
    let battery = match manager.into_iter().next() {
        Some(battery) => battery,
        None => {
            if !has_asked {
                println!("[{}] This benchmark is meant for laptops", "WARNING".red());
                println!("[{}] This benchmark will infinitely loop compiling something until it runs out of battery", "WARNING".red());
                print!("Would you like to continue anyway? [Y/N] ");
                stdout().flush().unwrap();
                let mut input = String::new();
                stdin().read_line(&mut input).unwrap();
                if input.trim().to_lowercase() != "y" {
                    exit(1);
                }
            }
            return false;
        },
    };
    let state = battery.unwrap().state();
    match state {
        State::Charging => { return true; },
        State::Full => { return true; },
        _ => { return false; },
    }
}

fn execute_build_command(command: &str) {
    let iterator = command.split_whitespace();
    let mut command = Command::new(iterator.clone().next().unwrap());
    for arg in iterator.skip(1) {
        command.arg(arg);
    }
    let mut process = command
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to build linux kernel");

    let reader = BufReader::new(process.stdout.take().expect("failed to get stdout"));
    for line in reader.lines() {
        match line {
            Ok(line) => {
                println!("{}", line);
            },
            Err(_) => {},
        }
    }
}

fn copy_directory(source: &str, destination: &str) -> std::io::Result<()> {
    create_dir_all(destination)?;

    for entry in read_dir(source)? {
        let entry = entry?;
        let entry_type = entry.file_type()?;
        let entry_path = entry.path();
        let destination_path = format!("{}/{}", destination, entry_path.file_name().unwrap().to_string_lossy());

        if entry_type.is_dir() {
            copy_directory(&entry_path.to_string_lossy(), &destination_path)?;
        } else {
            copy(&entry_path, &destination_path)?;
        }
    }

    Ok(())
}

