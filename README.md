# RustCord Red

A social/community product starter inspired by Discord + story feeds + voice rooms + reels. This repo now includes a real Rust Axum backend foundation with in-memory data for:

- authentication mock flow
- users
- stories
- DM messages
- group creation and data model
- voice room data
- reels / shorts demo
- search history
- call session tracking
- notifications

## Features included

- Rust + Axum server
- JSON API endpoints
- story posting with media support
- search history including duration and timestamps
- call tracking with room name and duration
- DM list and message flow
- group cards and voice room cards
- reels / shorts card demo
- notification list

## Local run

```bash
cargo run
```

Then open:

```text
http://localhost:3000
```

## Main API endpoints

- POST /api/login
- GET /api/state
- POST /api/search-history
- POST /api/stories
- POST /api/dm
- POST /api/groups
- POST /api/voice-rooms
- POST /api/reels
- POST /api/notifications
- POST /api/call-sessions

## Roadmap

The current repo is a working product scaffold and demo-logic foundation. The next production steps are:

1. Real Postgres persistence with SQLx
2. JWT auth and password hashing
3. WebSocket realtime updates
4. WebRTC voice and camera room implementation
5. Screen sharing
6. Role / permissions model for groups
7. YouTube API and media upload system
8. Deployment with Docker and environment configs
