# Efficiency Benchmark

## Description

This benchmark it meant to measure laptop cpu efficiency and poower consumption
by contineuosly compiling something until it runs out of battery.
The benchmark create a directory called "benchmark" where you runned the command
and stores this in theree:
   - The repository
   - A copy of the repo to build
   - A file containing the score


## Installation

```bash
cargo install efficiency-benchmark
```

## Usage

To run it with default options just run
```bash
efficiency-benchmark 
```


### Help command


```bash

$ efficiency-benchmark --help

Usage: efficiency-benchmark [OPTIONS]
            
    Options:
        --help, -h    Show this help message
        --repo, -r    Set the repository URL to clone
        --build, -b   Set the build command
    
    If no args provided it will use by default:
        repo-url   -> https://github.com/rust-lang/rustlings.git
        build-cmd  -> cargo build


    This benchmark is meant for laptops,
    It will infinitely loop compiling something until it runs out of battery
    That's how this benchmark works

```
