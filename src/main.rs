use std::env::set_current_dir;
use std::fs::{create_dir, read_dir, remove_file};
use std::io::{BufRead, BufReader, stdout, Write, stdin};
use std::process::{Command, exit, Stdio};
use std::thread::sleep;
use std::time::Duration;
use battery::{Manager, State};
use colored::Colorize;


fn main() {

    let repo_url = "https://github.com/hyprwm/Hyprland.git";
    let build_command = "make all";

    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        for arg in args.clone() {
            if arg == "--help" {
                println!("Usage: {} [repo-url] [build-command] [options]", args[0]);
                println!("");
                println!("Options:");
                println!("  --help: Display this help message");
                println!("");
                println!("Default options:");
                println!("  repo-url: {}", repo_url);
                println!("  build-command: {}", build_command);
                exit(0);
            }
        }
    }


    // check if system in plugged
    if is_plugged() {
        println!("Please unplug the system to start th benchmarking");
        loop {
            if !is_plugged(){
                break;
            }
            sleep(Duration::from_secs(1));
        }
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

    if is_plugged() {
        println!("Please unplug the system to start th benchmarking");
        loop {
            if !is_plugged(){
                break;
            }
            sleep(Duration::from_secs(1));
        }
    }

    // check if battery is full

    let mut hash_asked = false;
    // check if directory exists
    let dir = read_dir("benchmark");
    if dir.is_ok() {
        for entry in dir.unwrap() {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_dir() && !hash_asked {
                hash_asked = true;
                print!("Benchmark directory already exists, would you like to delete it? [Y/N] ");
                stdout().flush().unwrap();
                let mut input = String::new();
                stdin().read_line(&mut input).unwrap();
                if input.trim().to_lowercase() == "y" {
                    delete_dir("benchmark");
                    create_dir("benchmark").unwrap();
                }
            }
        }
    }else {
        create_dir("benchmark").unwrap();
    }
    set_current_dir("benchmark").unwrap();

    // delete log file
    let logfile = "benchmark-score.log";
    let log = read_dir(".");
    if log.is_ok() {
        for entry in log.unwrap() {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_file() {
                if entry.file_name().to_str().unwrap() == logfile {
                    remove_file(logfile).unwrap();
                }
            }
        }
    }

    let repo_dir = read_dir("repo_dir");
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
                    delete_dir("repo_dir");
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
            .arg("repo_dir")
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

    let copy_dir = read_dir("build-dir");


    if copy_dir.is_ok() {
        for entry in copy_dir.unwrap() {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_dir() {
                delete_dir("build-dir");
            }
        }
    }
    loop {
        // Copy the linux kernel to another directory
        println!("Copying repo");

        Command::new("cp")
            .arg("-r")
            .arg("repo_dir")
            .arg("build-dir")
            .stdout(Stdio::null())
            .output()
            .expect("failed to copy linux kernel");


        set_current_dir("build-dir").unwrap();


        // build the kernel

        // Linux:
        execute_build_command(build_command);

        // remove the linux kernel
        set_current_dir("../").unwrap();
        add_one();
        delete_dir("build-dir");
    }
}

fn delete_dir(path: &str) {
    Command::new("rm")
        .arg("-rf")
        .arg(path)
        .output()
        .expect("failed to delete directory");
}

fn add_one() {
    let logfile = "benchmark-score.log";


    let contents = Command::new("cat")
        .arg(logfile)
        .output()
        .expect("failed to create log file");

    let mut score = 0;
    if contents.stdout.len() > 0 {
        score = String::from_utf8_lossy(&contents.stdout).parse().unwrap();
    }
    score += 1;
    Command::new("sed")
        .arg("-i")
        .arg("s/.*/".to_owned() + &score.to_string() + "/")
        .arg(logfile)
        .output()
        .expect("failed to update log file");

    println!("Current Score: {}", score);
}

fn get_battery_percentage() -> u8 {
    let manager = Manager::new().unwrap().batteries().unwrap();
    let battery = manager.into_iter().next().unwrap().unwrap();
    let percentage = battery.state_of_charge().value * 100.0;
    return percentage as u8;
}

fn is_plugged() -> bool {
    let manager = Manager::new().unwrap().batteries().unwrap();
    let battery = manager.into_iter().next().unwrap().unwrap();
    let state = battery.state();
    return state == State::Charging;
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

