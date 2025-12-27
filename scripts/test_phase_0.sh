#!/bin/bash

# Test AI Inference (Spawn Blocking)
echo "Testing AI Inference (should take 2 seconds)..."
curl -X POST http://127.0.0.1:8082/api/ai/inference \
  -H "Content-Type: application/json" \
  -d '{"prompt": "Test Prompt"}'
echo -e "\nAI Test Complete.\n"

# Test Player Command (Bevy Bridge)
echo "Testing Player Command..."
curl -X POST http://127.0.0.1:8082/api/player/command \
  -H "Content-Type: application/json" \
  -d '{
    "command_text": "look",
    "current_character": {
        "id": "1",
        "name": "TestPlayer",
        "primary_archetype_id": null,
        "stats": {},
        "current_quest_id": "quest_001",
        "current_step_id": "step_001",
        "current_step_description": "Start",
        "inventory": [],
        "report_summaries": []
    }
  }'
echo -e "\nPlayer Command Test Complete."
