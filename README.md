<h1 align="center">
  <br>
  <img src="assets/logo.svg" alt="logss" width="400">
  <br>
  <br>
</h1>

<h5 align="center">logs splitter</h5>
<h4 align="center">A simple command line tool that helps you visualize an input stream of text.</h4>

![screenshot](./assets/gifs/complete.gif)

<p align="center">
  <img src="https://github.com/todoesverso/logss/actions/workflows/ci.yaml/badge.svg">
  <img src="https://img.shields.io/badge/PRs-welcome-brightgreen.svg?style=flat-square">
  <a href="https://codecov.io/gh/todoesverso/logss" >
    <img src="https://codecov.io/gh/todoesverso/logss/branch/main/graph/badge.svg?token=G6JEXYQQO0"/>
  </a>
</p>

<p align="center">
  <a href="#key-features">Key Features</a> •
  <a href="#usage">Usage</a> •
  <a href="#installation">Installation</a> •
  <a href="#download">Download</a> •
  <a href="#roadmap">Roadmap</a> •
  <a href="#license">License</a>
</p>

## Key Features

* Select render/stream speed
* Automatic color assigned to each string match
* Vertical and Horizontal view
* Pause and continue stream
* Scroll Up/Down
* Delete containers on runtime
* Add new containers on runtime
* Dedicated container for raw stream
* Toggle line wrapping
* Zoom into a specific container
* Containers Show/Hide 
* Support for regexp
* Support for configuration file
* Support for explicit command (no need to pipe into it)
* Send all matched lines to dedicated files
* Consolidated view with highlighted items
* Simple BarChart popup with counts
* Support to trigger shell commands (thru 'bin/sh') fir each match
  * The line matched can be replaced in the command to execute (__line__)
  * Timeout for each trigger
  * Configurable number of threads for each container


## Usage

  ```sh
  $ logss -h
  Simple CLI command to display logs in a user-friendly way

  Usage: logss [OPTIONS]

  Options:
    -c <CONTAINERS>  Specify substrings (regex patterns)
    -e               Exit on empty input [default: false]
    -s               Start in single view mode [default: false]
    -C <COMMAND>     Get input from a command
    -f <FILE>        Input configuration file (overrides CLI arguments)
    -o <OUTPUT_PATH> Specify the output path for matched patterns
    -r <RENDER>      Define render speed in milliseconds [default: 100]
    -t <THREADS>     Number of threads per container for triggers [default: 1]
    -V               Start in vertical view mode
    -h               Print help

  $ cat shakespeare.txt | logss -c to -c be -c or -c 'in.*of'
  $ # 
  $ cat real_curl_example.yaml
    command:
      - curl
      - -s
      - https://raw.githubusercontent.com/linuxacademy/content-elastic-log-samples/master/access.log
    render: 75
    containers:
      - re: GET
        trigger: echo $(date) >> /tmp/get.log
        timeout: 4
      - re: "404"
        trigger: echo __line__ >> /tmp/404.log
        timeout: 4
      - ".*ERROR|error.*"
  $ logss -f real_curl_example.yaml 
  ```

## Installation

So far only available in crates.io.

```shell
cargo install logss
```

If cargo is not a possibility then download pre compiled binaries from the [download](#download) section.

### Arch Linux (AUR)

You can install `logss` from the [AUR](https://aur.archlinux.org/packages/logss) with using an [AUR helper](https://wiki.archlinux.org/title/AUR_helpers).

```shell
paru -S logss
```

## Download

Pre compiled binaries for several platforms can be downloaded from the [release](https://github.com/todoesverso/logss/releases) section.

## Roadmap

This is just a personal project intended to learn Rust, so things move slowly. 

This is a list of things I plan to do:

* Add documentation (the rust way)
* Refactoring (as I learn more Rust things)
* Tests
* Smart timestamp highlights
* ... whatever I can think of when I am using it

## License

MIT
