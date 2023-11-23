# RAoC - Rust Advent of Code helper library

An oxidation of [`aoc_helper`](https://github.com/starwort/aoc_helper).

## Usage

RAoC is both a library and a binary. The binary can be used to control the configuration for the library (instead of modifying the configuration directory directly).

RAoC shares its configuration directory with `aoc_helper`; so if you have one working, the other should too.

## Configuration

*(lifted straight from the documentation of `aoc_helper`)*

> When you first use any function that interfaces with Advent of Code, you will be prompted to enter your session token.
>
> Your session token is stored as a *HTTPOnly cookie*. This means there is no way of extracting it with JavaScript, you either must
> use a browser extension such as [EditThisCookie](http://www.editthiscookie.com/), or follow [this guide](https://github.com/wimglenn/advent-of-code-wim/issues/1)
>
> This token is stored in `~/.config/aoc_helper/token.txt` (`C:\Users\YOUR_USERNAME\.config\aoc_helper\token.txt` on Windows),
> and other `aoc_helper` data is stored in this folder (such as your input and submission caches).
>
> If, for whatever reason, you feel the need to clear your caches, you can do so by deleting the relevant folders in `aoc_helper`'s
> configuration folder.
