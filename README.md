# Deletion सर्वोपरि 

A bot that celebrates deletion of code.

This hastily written "script" uses [egg-mode] and [octocrab] to tweet out
the Github push events that delete code. Yes, the author is aware of
the irony in writing more code to celebrate the deletion of code. That
is the point.

Tweets out to when running https://twitter.com/delservopari but you can
change it with your tokens.

## Requirements

The following environment variables must be set.

1. `GITHUB_TOKEN`
2. `TWITTER_ACCESS_KEY`
3. `TWITTER_API_KEY`
4. `TWITTER_ACCESS_SECRET`
5. `TWITTER_API_SECRET`

## Build

`cargo build`

## Tests

LOL

## Run

`cargo run`


[egg-mode]: https://github.com/egg-mode-rs/egg-mode
[octocrab]: https://github.com/XAMPPRocky/octocrab