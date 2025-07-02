# QWOT

Print a random quote from [This Week In Rust](https://this-week-in-rust.org/).

## Installation

```
cargo install --git https://github.com/tatounee/qwot.git
```

## Usage

Print a random quote.
```
qwot
```

Fetch quote from new issues.
```
qwot -f
```
You can create a `crontab` to fecth new quotes each week.

## Storage

All quotes are stored in `$XDG_DATA_HOME/qwot/quotes` or `$HOME/.local/share/qwot/quotes`.
