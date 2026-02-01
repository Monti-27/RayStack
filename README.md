# RayStack

A fast, no-nonsense monitor for Pump.fun on Solana.

I built this because I needed a way to catch new token launches immediately without the lag you get from standard RPC polling. It connects directly to a Solana WebSocket, filters the logs for Pump.fun's "Create" instruction, and pipes the alerts straight to Discord.

It's written in Rust so it just runs in the background without eating up memory.

## How it works

The logic is pretty simple: it's a producer-consumer setup. One part waits for the logs, passes them to a channel, and the other part processes them and handles the alerts.

```mermaid
graph TD
    subgraph Solana ["Solana Network"]
        RPC[("RPC Node")]:::sol
    end

    subgraph App ["RayStack Engine"]
        L["Listener"]:::comp
        B{{"Buffer"}}:::comp
        H["Handler"]:::comp
    end

    subgraph Out ["Output"]
        D["Discord"]:::dis
    end

    RPC == "Logs (WSS)" ==> L
    L -- "Filtered Events" --> B
    B -- "Async Data" --> H
    H -- "New Token Alert" --> D

    classDef sol fill:#000,stroke:#14F195,color:#fff,stroke-width:2px;
    classDef comp fill:#111,stroke:#9945FF,color:#fff,stroke-width:2px;
    classDef dis fill:#5865F2,stroke:#fff,color:#fff,stroke-width:2px;
```

## Features

*   **Fast**: Uses WSS logs subscription, so no polling delay.
*   **Specific**: Only looks for the `Create` instruction on the Pump.fun program (`6EF8...`).
*   **Async**: The listener never blocks. It dumps events into a buffer and keeps listening while the processor handles the Discord API calls.
*   **Resilient**: If the socket drops, it just reconnects automatically.

## Demo

### Terminal Logs
Clean, human-readable logs showing accepted and rejected tokens.
![Terminal Logs](assets/terminal.png)

### Discord Alerts
Rich embeds with Ticker, Name, Dev Spend, and quick links.
![Discord Alerts](assets/discord.png)

## Setup

1.  Clone the repo.
2.  Make a `.env` file:

    ```bash
    RPC_URL=wss://api.mainnet-beta.solana.com
    DISCORD_WEBHOOK=https://discord.com/api/webhooks/...
    ```

3.  Run it:

    ```bash
    cargo run --release
    ```

That's it. It'll start printing logs when it finds new tokens.
