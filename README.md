# LynxLevin's LifeTracker (Actix backend)

## Motivation
I wanted to create a goal managing app for myself. By a quick survey around the existing apps, most of them hold either one or two levels of concepts, Goals and Actions. I wanted something different, an app with three levels of concepts, Ambitions (ultimate goals to achieve), DesiredStates (small goals that lead you closer to achieving your ambitions) and Actions.
These three concepts are not yet finalized, I’m looking for an ideal set of concepts to support, while actively using and upgrading this app by myself.

Use this along with [lllifetracker-frontend](https://github.com/lynxlevin/lllifetracker-frontend)

## Setup
1. Clone this repository to local.
2. Setup Postgres server.
3. Setup Redis server.
4. `cp .env.example .env`
5. Fill in .env file.
6. `cargo run`
