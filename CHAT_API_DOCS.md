# Chat API Documentation

This document describes the new chat functionality that allows users to interact with the RAG system through conversational interfaces.

## Overview

The chat system provides:
- Session management for maintaining conversation context
- Chat history storage and retrieval
- Integration with vector search for relevant document retrieval
- AI-powered responses using Google Gemini

## API Endpoints

### 1. Create Session
**POST** `/api/chat/session`

Creates a new chat session. Sessions are used to group related conversations.

**Response:**
```json
{
  "success": true,
  "message": "Session created successfully",
  "data": {
    "session_id": "uuid",
    "created_at": "2024-01-01T00:00:00Z"
  }
}
```

### 2. Chat with Bot
**POST** `/api/chat`

Main endpoint for chatting with the bot. Automatically creates sessions and chats if not provided.

**Request Body:**
```json
{
  "chatbot_id": "uuid",           // Required: ID of the chatbot
  "query": "string",              // Required: User's question
  "session_id": "uuid",          // Optional: Existing session ID
  "chat_id": "uuid"              // Optional: Existing chat ID
}
```

**Response:**
```json
{
  "success": true,
  "message": "Chat request processed successfully",
  "data": {
    "session_id": "uuid",
    "chat_id": "uuid",
    "conversation_id": "uuid",
    "user_query": "string",
    "bot_response": "string",
    "context_used": ["file1.pdf", "file2.pdf"]
  }
}
```

### 3. Get Chat History
**GET** `/api/chat/history?chat_id=uuid`

Retrieves the conversation history for a specific chat.

**Response:**
```json
{
  "success": true,
  "message": "Chat history retrieved successfully",
  "data": {
    "chat_id": "uuid",
    "conversations": [
      {
        "id": "uuid",
        "sequence_number": 1,
        "user_query": "string",
        "bot_response": "string",
        "created_at": "2024-01-01T00:00:00Z"
      }
    ],
    "count": 1
  }
}
```

### 4. Health Check
**GET** `/api/chat/health`

Returns the health status of the chat service.

## Usage Examples

### Example 1: First-time User (No Session)
```bash
curl -X POST http://localhost:8000/api/chat \
  -H "Content-Type: application/json" \
  -d '{
    "chatbot_id": "your-chatbot-id",
    "query": "What is this document about?"
  }'
```

This will:
1. Create a new session automatically
2. Create a new chat within that session
3. Search for relevant documents
4. Generate an AI response
5. Store the conversation

### Example 2: Continuing a Conversation
```bash
curl -X POST http://localhost:8000/api/chat \
  -H "Content-Type: application/json" \
  -d '{
    "chatbot_id": "your-chatbot-id",
    "query": "Can you tell me more about the main topics?",
    "session_id": "existing-session-id",
    "chat_id": "existing-chat-id"
  }'
```

This will:
1. Use the existing session and chat
2. Include conversation history in the context
3. Search for relevant documents
4. Generate a contextual response

### Example 3: Getting Chat History
```bash
curl "http://localhost:8000/api/chat/history?chat_id=your-chat-id"
```

## Environment Variables

Make sure to set the following environment variables:

```bash
GEMINI_API_KEY=your_gemini_api_key_here
GEMINI_MODEL=gemini-1.5-flash  # Optional, defaults to gemini-1.5-flash
```

## Database Schema

The chat system uses the following database tables:

- **sessions**: Stores chat sessions
- **chats**: Stores individual chats within sessions
- **conversations**: Stores individual user-bot exchanges

## How It Works

1. **Query Processing**: User sends a query to the chat endpoint
2. **Session Management**: System creates or retrieves session/chat
3. **Vector Search**: Query is embedded and searched against stored documents
4. **Context Building**: Relevant documents and conversation history are combined
5. **AI Generation**: Gemini AI generates a response based on the context
6. **Storage**: Conversation is stored in the database
7. **Response**: Structured response is returned to the user

## Error Handling

The API returns appropriate HTTP status codes:
- `200`: Success
- `400`: Bad Request (invalid parameters)
- `404`: Not Found (session/chat not found)
- `500`: Internal Server Error

## Testing

Use the provided test script to test the chat functionality:

```bash
python test_chat_api.py
```

Make sure to:
1. Update the `CHATBOT_ID` in the test script
2. Set up your `GEMINI_API_KEY` environment variable
3. Ensure the server is running on `localhost:8000`
