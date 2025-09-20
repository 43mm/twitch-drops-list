# Twitch Drops List

A rust script to generate an organised list of active Twitch drop campaigns.

## Features

- Fetch all currently active Twitch drops campaigns from https://twitch-drops-api.sunkwi.com/drops
- Generate two lists:
  - Campaigns started in the last 7 days, sorted by date (most recent first) then by game
  - All active campaigns for each game
- Github action to run the script daily and publish the list to the drops branch