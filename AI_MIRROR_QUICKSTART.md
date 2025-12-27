# AI as a Mirror - Quick Start Guide

## Overview

This guide helps you get the AI Mirror system up and running. Follow these steps in order.

---

## Prerequisites

- ✅ Rust 1.70+ installed
- ✅ PostgreSQL 14+ running
- ✅ ~3GB free disk space (for model)
- ✅ GPU optional but recommended (4GB+ VRAM)

---

## Step 1: Apply Database Migration

```bash
cd backend

# Ensure DATABASE_URL is set
export DATABASE_URL="postgres://postgres:password@localhost:5432/daydream"

# Run migration
sqlx migrate run
```

**Expected output**:

```
Applied 001/create_conversation_turns (...)
```

---

## Step 2: Download AI Model

**Option A: Llama 3.2 3B (Recommended)**

```bash
# Install Hugging Face CLI
pip install huggingface-hub[cli]

# Login (requires free HF account)
huggingface-cli login

# Download model to project directory
cd ..  # Back to project root
mkdir -p models

huggingface-cli download meta-llama/Llama-3.2-3B-Instruct \
  --local-dir ./models/ \
  --include "*.gguf" "tokenizer.json"
```

**Option B: TinyLlama 1.1B (Faster, smaller)**

```bash
huggingface-cli download TinyLlama/TinyLlama-1.1B-Chat-v1.0 \
  --local-dir ./models/ \
  --include "*.gguf" "tokenizer.json"
```

**Verify**:

```bash
ls -lh models/
# Should see: *.gguf (~2GB) and tokenizer.json
```

---

## Step 3: Build Backend

```bash
cd backend
cargo build --release
```

**This will take 5-10 minutes** on first build (compiling Candle).

---

## Step 4: Run Backend

```bash
cargo run --release
```

**Expected output**:

```
Starting Daydream Backend Server...
DATABASE_URL found, connecting to the database...
Backend listening on http://0.0.0.0:3000
```

---

## Step 5: Test AI Mirror API

In a new terminal:

```bash
# Create a session
curl -X POST http://localhost:3000/api/ai-mirror/create-session

# Copy the session_id from response, then:
SESSION_ID="<paste-session-id-here>"

# Send a message
curl -X POST http://localhost:3000/api/ai-mirror/send-message \
  -H "Content-Type: application/json" \
  -d "{
    \"session_id\": \"$SESSION_ID\",
    \"user_id\": 1,
    \"message\": \"I'm feeling stuck on this reflection about my learning journey.\"
  }"
```

**Expected response**:

```json
{
  "ai_response": "I hear that you're feeling uncertain. When you think about your learning journey, what image or feeling comes to mind first?",
  "session_id": "550e8400-..."
}
```

---

## Troubleshooting

### "Model file not found"

**Issue**: `Llama3Model::load()` fails.

**Fix**: Verify model exists:

```bash
ls models/*.gguf
```

If missing, re-run Step 2.

---

### "Failed to connect to database"

**Issue**: PostgreSQL not running or wrong `DATABASE_URL`.

**Fix**:

```bash
# Check PostgreSQL is running
pg_isready

# Verify environment variable
echo $DATABASE_URL
```

---

### "AI response is placeholder text"

**Issue**: Model not fully loaded yet (Phase 1 limitation).

**Fix**: This is expected until full GGUF loading is implemented. The API structure works, inference is TODO.

---

### Compilation errors for Candle

**Issue**: Missing system dependencies (CUDA, etc.).

**Fix**: Candle will gracefully fall back to CPU if GPU libraries aren't found. Ignore warnings about CUDA/Metal.

---

## Next Steps

1. **Build Frontend UI** (Phase 1 task)
   - Create `frontend/src/components/ai_mirror.rs`
   - Chat-style interface
   - Connect to `/api/ai-mirror/*` endpoints

2. **Test Full Conversation** (Phase 1 completion criteria)
   - Have 5-turn dialogue
   - Verify AI asks questions (not gives answers)
   - Checklisted in walkthrough

3. **Phase 2: Prompt Engineering**
   - User testing sessions
   - Refine Socratic templates
   - Improve strategy selection

---

## Quick Reference

### API Endpoints

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/api/ai-mirror/create-session` | POST | Start new conversation |
| `/api/ai-mirror/send-message` | POST | Send message, get AI response |
| `/api/ai-mirror/get-history` | POST | Retrieve past turns |

### Project Structure

```
backend/
├── src/ai/              # ← AI Mirror code
│   ├── socratic_engine.rs
│   ├── conversation_memory.rs
│   ├── llm/llama_engine.rs
│   └── prompts/mod.rs
└── migrations/
    └── 001_create_conversation_turns.sql

models/                  # ← Download here
└── *.gguf
```

### Logs

```bash
# Backend logs show:
cargo run 2>&1 | grep -i "socratic\|llama\|ai"
```

---

For detailed architecture explanation, see `ai_mirror_phase1_walkthrough.md`.
