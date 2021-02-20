# Gitty

[![](https://img.shields.io/crates/v/gitty)](https://crates.io/crates/gitty)
![](https://img.shields.io/crates/l/gitty.svg)

Interactive command line interface for enhanced workflow when using the Git CLI.

## How To Use
You run `gitty` simply by calling `gitty` from the command line, no arguments, no bells and whistles. This will prompt you as to which `git` command you would like execute. 
```
$ gitty
What would you like to do?:
> status
  log
  diff
  stage
  commit
  checkout
  branch
  push
  pull
  exit
```

## Install 
You can install `gitty` from `crates.io` using
```
cargo install gitty
```
Alternatively, you can build locally with `rust` by cloning this repo and using `cargo run` from within the `gitty` directory
