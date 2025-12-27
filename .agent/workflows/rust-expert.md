---
description: Query llama_llama AI assistant for Rust coding help
---

# Consult Rust Expert (Overthinking Rustacean)

Use this workflow when you need expert Rust advice from the local Overthinking Rustacean model running on llama_llama server.

## Prerequisites

- llama_llama server running on port 8081
- Start with: `cd /home/joshua/antigravity/llama_llama && ./run.sh`

## How to Query

// turbo

1. Check if server is running:

```bash
curl -s http://localhost:8081/health | grep -q "ready" && echo "Ready" || echo "Server not running"
```

1. Ask a Rust question:

```bash
curl -s -X POST http://localhost:8081/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{"messages":[{"role":"user","content":"YOUR_QUESTION_HERE"}],"max_tokens":1024}'
```

## Use Cases

- **Code Review**: Ask for review of Rust code snippets
- **Error Help**: Paste compiler errors for explanations
- **Design Advice**: Get architectural guidance for Rust systems
- **Optimization**: Ask about performance improvements

## Example Queries

### Debug a borrow checker error

```bash
curl -s -X POST http://localhost:8081/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{"messages":[{"role":"user","content":"Why does this Rust code fail: let x = vec![1,2,3]; let y = x; println!(\"{:?}\", x);"}],"max_tokens":512}'
```

### Get implementation advice

```bash
curl -s -X POST http://localhost:8081/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{"messages":[{"role":"user","content":"How should I implement a thread-safe cache in Rust?"}],"max_tokens":1024}'
```

## Response Format

The model returns JSON with:

- `choices[0].message.content` - The response (may include `<think>` reasoning)
- `usage.total_tokens` - Token count

## Notes

- Model is Overthinking-Rustacean (77B params, Q4_K_M quantization)
- Uses `<think>` tags for chain-of-thought reasoning
- Running on Strix Halo with Vulkan GPU acceleration
- ~44GB VRAM usage
