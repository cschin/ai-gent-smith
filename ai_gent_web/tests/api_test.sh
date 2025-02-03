curl -X POST \
  -H "Content-Type: application/json" \
  -d '{
    "text": "This is a sample text for embedding.",
    "max_tokens": 1000,
    "chunk_tokens": 200,
    "overlapped_tokens": 50
  }' \
  http://127.0.0.1:8080/api/service/text_to_embedding
