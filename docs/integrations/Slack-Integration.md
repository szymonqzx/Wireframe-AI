# Slack Integration Guide

Wireframe-AI integrates with Slack through the `wireframe-ai-integrations-core` module.

## Setup

1. Create a Slack app with Bot Token Scopes: `chat:write`, `channels:read`.

2. Configure the integration:

```bash
nats pub integration.config.set '{
  "service": "slack",
  "api_key": "xoxb-your-bot-token",
  "base_url": "https://slack.com/api"
}'
```

## Available Actions

### Send Message

```bash
nats pub integration.slack.message.send '{
  "channel": "#alerts",
  "text": "Deployment completed successfully"
}'
```

### List Channels

```bash
nats pub integration.slack.channels.list '{}'
```

## Webhooks

Use the `wireframe-ai-webhooks-core` module to receive Slack events:

```bash
nats pub webhook.configure '{
  "source": "slack_events",
  "target_topic": "slack.events.received"
}'
```
