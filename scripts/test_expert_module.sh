#!/bin/bash

# Test Expert Module (Graph API)
echo "Testing Expert Module API..."

# 1. Get Graph (Should return default or existing graph)
echo "GET /api/expert/graph"
curl -v -s http://127.0.0.1:8082/api/expert/graph 2>&1 | grep "Demo Story" && echo "GET Success" || echo "GET Failed (or DB empty/server not running)"

# 2. Save Graph (Update title)
echo "POST /api/expert/graph"
curl -v -s -X POST http://127.0.0.1:8082/api/expert/graph \
  -H "Content-Type: application/json" \
  -d '{
    "id": "demo_graph",
    "title": "Updated Demo Story",
    "nodes": [
        {
            "id": "node_1",
            "title": "Test Node",
            "content": "Content",
            "x": 100.0,
            "y": 100.0
        }
    ],
    "connections": []
  }' 2>&1 | grep "Updated Demo Story" && echo "POST Success" || echo "POST Failed"

# 3. Verify Persistence (Get again)
echo "GET /api/expert/graph (Verify Persistence)"
curl -s http://127.0.0.1:8082/api/expert/graph | grep "Updated Demo Story" && echo "Persistence Success" || echo "Persistence Failed"

echo -e "\nExpert Module Test Complete."
