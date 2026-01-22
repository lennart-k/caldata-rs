[![license](http://img.shields.io/badge/license-Apache%20v2-orange.svg)](https://raw.githubusercontent.com/lennart-k/caldata-rs/main/LICENSE)
[![Coverage Status](https://coveralls.io/repos/github/lennart-k/caldata-rs/badge.svg?branch=main)](https://coveralls.io/github/lennart-k/caldata-rs?branch=main)
[![Latest version](https://img.shields.io/crates/v/caldata.svg)](https://crates.io/crates/caldata)
[![Documentation](https://docs.rs/caldata/badge.svg)](https://docs.rs/caldata)

# caldata-rs

> [!NOTE]
> This package started as a fork of [ical-rs](https://github.com/Peltoche/ical-rs) aiming to add some more validation and accessors to calendar components.
> Since the original repository is archived and this version has diverged quite significantly, I am going to maintain this as a hard fork.
> The main difference to the original project is that this version is very strict in enforcing the iCalendar spec and in turn also parses data types like date-times and offers methods for recurrence expansion.
> Thanks [Peltoche](https://github.com/Peltoche) for providing a solid codebase to build upon.

## Installing

Put this in your `Cargo.toml`:

```toml
[dependencies]
caldata = "0.12"
```
