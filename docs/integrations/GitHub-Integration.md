# GitHub Integration Guide

Wireframe-AI integrates with GitHub through the `wireframe-ai-integrations-core` module.

## Setup

1. Configure the integration module with your GitHub API token:

```bash
nats pub integration.config.set '{
  "service": "github",
  "api_key": "ghp_your_token_here",
  "base_url": "https://api.github.com"
}'
```

2. Verify configuration:

```bash
nats sub integration.response
```

## Available Actions

### List Issues

```bash
nats pub integration.github.issues.list '{
  "owner": "wireframe-ai",
  "repo": "wireframe-ai",
  "state": "open"
}'
```

### Create Issue

```bash
nats pub integration.github.issues.create '{
  "owner": "wireframe-ai",
  "repo": "wireframe-ai",
  "title": "Bug: something broke",
  "body": "Description here"
}'
```

### List Pull Requests

```bash
nats pub integration.github.pr.list '{
  "owner": "wireframe-ai",
  "repo": "wireframe-ai",
  "state": "open"
}'
```

## Agent Usage

Agents can invoke GitHub actions via tool calls. The adapter will publish to the appropriate `integration.github.>` topic and wait for `integration.result`.
