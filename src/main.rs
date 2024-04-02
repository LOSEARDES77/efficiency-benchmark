use std::env::{args, set_current_dir};
use std::fs::{copy, create_dir, create_dir_all, metadata, read_dir, remove_dir_all, remove_file, File};
use std::io::{BufRead, BufReader, stdout, Write, stdin};
use std::process::{Command, exit, Stdio};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::{self, sleep};
use std::time::Duration;
use battery::{Manager, State};
use colored::Colorize;
use chrono::Local;


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

/// Runs the benchmark.
///
/// This method takes in the repository URL, build command, source directory, build directory, and a flag indicating whether the repository already exists.
/// It creates a channel for communication between threads and spawns a new thread to perform the benchmarking.
/// If the repository does not exist, it clones the repository using `git clone` command.
/// It then checks if the build directory exists and removes it if it does.
/// If the system is plugged in, it waits for the system to be unplugged before starting the benchmarking.
/// It creates a log file with a timestamp and starts an infinite loop.
/// In each iteration of the loop, it copies the source directory to the build directory, changes the current directory to the build directory, executes the build command, changes the current directory back, removes the build directory, and increments the score in the log file.
/// The output of each step is sent through the channel to be consumed by the main thread.
/// The method returns an iterator over the output lines received from the channel.
pub fn bench(repo_url: &str, build_command: &str, source_dir: &str, build_dir: &str, repo_exists: bool) -> impl Iterator<Item = String> {
    let (sender, receiver): (Sender<String>, Receiver<String>) = channel();
    let repo_url = repo_url.to_owned();
    let build_command = build_command.to_owned();
    let source_dir = source_dir.to_owned();
    let build_dir = build_dir.to_owned();
    thread::spawn(move || {
        if !repo_exists {
            let mut command = Command::new("git")
                .arg("clone")
                .arg("--progress")
                .arg(&repo_url)
                .arg("--recursive")
                .arg(&source_dir)
                .stdout(Stdio::piped())
                .spawn()
                .expect("failed to clone repository");
    
            let reader = BufReader::new(command.stdout.take().expect("failed to get stderr"));
            for line in reader.lines() {
                match line {
                    Ok(line) => {
                        sender.send(line.clone()).unwrap(); // Add the output line to the vector
                    },
                    Err(_) => {},
                }
            }
        
            command.wait().expect("failed to wait for command");
        }
    
        if metadata(&build_dir).is_ok() {
            remove_dir_all(&build_dir).unwrap();
        }
    
        
        if is_plugged(false) {
            sender.send("Please unplug the system to start the benchmarking".to_string()).unwrap();
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
            sender.send("Copying repo".to_string()).unwrap();
            copy_directory(&source_dir, &build_dir).expect("failed to copy src directory");
    
            set_current_dir(&build_dir).unwrap();
            
            // Build
            sender.send("Building".to_string()).unwrap();
            execute_build_command(&build_command);
    
            // Delete build dir
            set_current_dir("../").unwrap();
            remove_dir_all(&build_dir).unwrap();
            
            // Add score
            sender.send("Build successful!".to_string()).unwrap();
            
            add_one(logfile);
        }
    });

    receiver.into_iter()
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

/// Returns the battery percentage
/// If on a device without battery, it will return 100
pub fn get_battery_percentage() -> u8 {
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

/// Returns true if the laptop is plugged in
/// Returns false if the laptop is not plugged in
/// If on a device without battery, it will ask the user if they want to continue
/// and return false if the user confirms
pub fn is_plugged(has_asked: bool) -> bool {
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

pub fn execute_build_command(command: &str) {
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

