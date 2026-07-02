# game/relay

Bun WebSocket relay server for board game multiplayer. Connects two browser
peers in a named session and forwards game messages verbatim between them.
No game logic runs here — it's a pure pipe.

## Local dev

Requires [Bun](https://bun.sh) — install with `curl -fsSL https://bun.sh/install | bash`.

```sh
cd game/relay
bun run dev      # starts on :3001 with --watch
```

The frontend defaults to `ws://localhost:3001/ws` when `VITE_RELAY_URL` is not set,
so no `.env` file is needed for local development.

## Deploy to Fly.io

Costs ~$1–2/month with `auto_stop_machines` (machine stops when idle, you pay for runtime only).

1. Create an account at https://fly.io (credit card required).
2. Install the CLI: `brew install flyctl` (macOS) or see https://fly.io/docs/hands-on/install-flyctl/
3. Log in: `fly auth login`
4. First deploy:
   ```sh
   cd game/relay
   fly launch --name boardgame-relay --region fra --no-deploy
   fly deploy
   ```
5. Note the app URL (e.g. `https://boardgame-relay.fly.dev`).
6. Set the relay URL for production builds:
   - In `game/frontend/.env.production` (gitignored):
     ```
     VITE_RELAY_URL=wss://boardgame-relay.fly.dev/ws
     VITE_RELAY_HTTP_URL=https://boardgame-relay.fly.dev
     ```
   - Or as GitHub Actions secrets `VITE_RELAY_URL` / `VITE_RELAY_HTTP_URL` if building in CI.

## Subsequent deploys

```sh
cd game/relay
fly deploy
```

## Endpoints

- `GET /probe/:code` — liveness probe. Returns `{"live":true,"paired":bool}` (200)
  or `{"live":false}` (404). Used by the lobby's session status dots.
- `WS /ws` — all session traffic. Peers send relay envelopes (`{"type":"create"}` etc.)
  then game messages which are forwarded verbatim.
