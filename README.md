# rss-notify

Schedule system notifications for your favorite RSS feeds!

<img width="313" height="170" alt="image" src="https://github.com/user-attachments/assets/890f655d-20da-4d8b-927b-c2b64c473f8d" />

Created by: Andrew Martinez

## Description

`rss-notify` is a lightweight RSS notification tool with the ability to check your subscribed feeds and can alert you when new items are available.

It supports customizable check intervals per feed, tracks what you've already seen, and integrates with desktop notifications so you never miss updates.

Designed to be minimal, fast, and easy to configure using a simple TOML file.

IT is designed to be run externally - I will personally call it during my startup script.

## Status

`Version 0.5.0` -> Using the CLI, you can add, remove, view feeds from the config and you can check if any feed has new items based off your requested frequency.

## Features (Planned)

- Support for multiple RSS feeds - Done ✅
- Configurable notification schedule - Done ✅
- Cross-platform system notifications - Done ✅
- Option to mark items as read or ignored - WIP

### Next Steps

- More comments
- Improved error handling
- Ability to see the list off unseen items within the CLI with the URL to view them from the browser
