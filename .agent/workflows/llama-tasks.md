---
description: Run llama_llama autonomous task queue
---

# Autonomous llama_llama Task Runner

This workflow enables llama_llama to process tasks independently and generate reports for follow-up.

## Quick Start

// turbo

1. Check server status:

```bash
cd /home/joshua/antigravity/llama_llama && ./task_runner.sh status
```

// turbo
2. Start processing queued tasks:

```bash
cd /home/joshua/antigravity/llama_llama && ./task_runner.sh process
```

## Adding Tasks

1. Add a new task to the queue:

```bash
./task_runner.sh add "Your prompt here" 1024
```

Or edit the queue file directly:

```bash
vim /home/joshua/antigravity/llama_llama/tasks/queue.json
```

## Monitoring

// turbo
4. Check completed results:

```bash
ls -la /home/joshua/antigravity/llama_llama/tasks/results/
```

// turbo
5. View a specific result:

```bash
cat /home/joshua/antigravity/llama_llama/tasks/results/TASK_ID.json | jq '.response'
```

## Generating Reports

// turbo
6. Generate a summary report for Gemini:

```bash
./task_runner.sh report
```

## Watch Mode (Continuous)

1. Run in watch mode (processes every 10s):

```bash
./task_runner.sh watch
```

## File Locations

- **Queue**: `tasks/queue.json`
- **Results**: `tasks/results/*.json`
- **Log**: `tasks/runner.log`
- **Reports**: `tasks/report_*.md`
