# RAoC - Rust Advent of Code helper library

An oxidation of [`aoc_helper`](https://github.com/starwort/aoc_helper).

## Usage

RAoC is both a library and a binary. The binary can be used to control the configuration for the library (instead of modifying the configuration directory directly).

RAoC shares its configuration directory with `aoc_helper`; so if you have one working, the other should too.

## Automation

This project aims to be compliant with the [Advent of Code Automation Guidelines](https://www.reddit.com/r/adventofcode/wiki/faqs/automation). Here are the strategies it uses:

- Once inputs are downloaded, they are cached in `~/.config/aoc_helper/YEAR/DAY.in` (or a similar path for Windows users) - [`sync_fetch`](https://github.com/Starwort/raoc/blob/master/src/sync_impl/interface.rs#L128-L134), [`async_fetch`](https://github.com/Starwort/raoc/blob/master/src/async_impl/interface.rs#L144-L151)
- The `User-Agent` header declares the package name, version, and my contact info - [`USER_AGENT`](https://github.com/Starwort/raoc/blob/master/src/data.rs#L48-L53), used for every [sync](https://github.com/Starwort/raoc/blob/master/src/sync_impl/internal_util.rs#L130-L135) and [async](https://github.com/Starwort/raoc/blob/master/src/async_impl/internal_util.rs#L132-L137)
- If requesting input before the puzzle unlocks, the library will wait for unlock before sending any requests (except on day 1, where it will send a request to validate the session token) - [sync](https://github.com/Starwort/raoc/blob/master/src/sync_impl/interface.rs#L100-L113), [async](https://github.com/Starwort/raoc/blob/master/src/async_impl/interface.rs#L111-L127)
- If sending an answer too soon after an incorrect one, [the library will wait the cooldown specified in the response](https://github.com/Starwort/raoc/blob/master/src/sync_impl/interface.rs#L281) ([async](https://github.com/Starwort/raoc/blob/master/src/async_impl/interface.rs#L322)) (sending only one extra request; it *is* however possible for a user to send multiple requests in quick succession, by repeatedly calling `submit` before the cooldown is over)
- Advent of Code will not be queried at all [if the puzzle has already been solved](https://github.com/Starwort/raoc/blob/master/src/sync_impl/interface.rs#L237-L240) ([async](https://github.com/Starwort/raoc/blob/master/src/async_impl/interface.rs#L273-L278)) or [if an answer has already been submitted](https://github.com/Starwort/raoc/blob/master/src/sync_impl/interface.rs#L241-L250) ([async](https://github.com/Starwort/raoc/blob/master/src/async_impl/interface.rs#L279-L288))
<!-- - If, for some reason, the user decides they wish to clear their cache (for example, if they believe their input to be corrupted) they can do so by using the [`aoc clean`](https://github.com/Starwort/aoc_helper/blob/master/aoc_helper/main.py#L91-L121) command. -->

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
