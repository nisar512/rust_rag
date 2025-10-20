# RAG Chat Backend API Documentation

## Overview

This document provides comprehensive API documentation for integrating frontend applications with the RAG (Retrieval-Augmented Generation) Chat Backend. The backend supports both regular chat and streaming chat functionality.

## Base URL

```
http://localhost:8000/api
```

## Authentication

Currently, no authentication is required. All endpoints are publicly accessible.

## Content Types

- **Request**: `application/json`
- **Response**: `application/json` (regular endpoints) or `text/event-stream` (streaming endpoints)

---

## Endpoints

### 1. Health Check

**GET** `/health`

Check if the server is running.

**Response:**
```json
{
  "status": "ok",
  "message": "RAG Server is running",
  "timestamp": "2024-01-01T00:00:00Z"
}
```

### 2. Chat Service Health

**GET** `/chat/health`

Check if the chat service is running.

**Response:**
```json
{
  "status": "ok",
  "message": "Chat service is running",
  "timestamp": "2024-01-01T00:00:00Z"
}
```

### 3. Get All Chatbots

**GET** `/chatbots`

Retrieve all available chatbots.

**Response:**
```json
{
  "success": true,
  "message": "Chatbots retrieved successfully",
  "data": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "name": "My Knowledge Bot",
      "created_at": "2024-01-01T00:00:00Z",
      "updated_at": "2024-01-01T00:00:00Z",
      "status": "active"
    }
  ],
  "count": 1
}
```

### 4. Regular Chat

**POST** `/chat`

Send a message to the chatbot and receive a complete response.

**Request Body:**
```json
{
  "chatbot_id": "string (required)",
  "query": "string (required)",
  "session_id": "string (optional)",
  "chat_id": "string (optional)"
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

**Example Request:**
```bash
curl -X POST http://localhost:8000/api/chat \
  -H "Content-Type: application/json" \
  -d '{
    "chatbot_id": "b43c1d87-789e-46d2-aa0a-de1a5a8a1092",
    "query": "What is this document about?"
  }'
```

### 5. Streaming Chat

**POST** `/chat/stream`

Send a message to the chatbot and receive streaming responses via Server-Sent Events (SSE).

**Request Body:**
```json
{
  "chatbot_id": "string (required)",
  "query": "string (required)",
  "session_id": "string (optional)",
  "chat_id": "string (optional)"
}
```

**Response Format:** Server-Sent Events (SSE)

Each event contains:
```
data: {"text": "chunk of response", "is_final": false, "session_id": "uuid", "chat_id": "uuid", "conversation_id": "uuid"}
```

**Example JavaScript Integration:**
```javascript
function startStreamingChat(chatbotId, query, sessionId = null, chatId = null) {
  const eventSource = new EventSource('/api/chat/stream', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      chatbot_id: chatbotId,
      query: query,
      session_id: sessionId,
      chat_id: chatId
    })
  });

  eventSource.onmessage = function(event) {
    const data = JSON.parse(event.data);
    
    // Display the chunk
    displayMessageChunk(data.text);
    
    // Check if streaming is complete
    if (data.is_final) {
      eventSource.close();
      onStreamingComplete(data);
    }
  };

  eventSource.onerror = function(event) {
    console.error('Streaming error:', event);
    eventSource.close();
  };
}
```

### 6. Create Session

**POST** `/chat/session`

Create a new chat session.

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

### 7. Get Chat History

**GET** `/chat/history?chat_id={chat_id}`

Retrieve conversation history for a specific chat.

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

---

## Frontend Integration Guide

### 1. Basic Chat Implementation

```javascript
class RAGChatClient {
  constructor(baseUrl = 'http://localhost:8000/api') {
    this.baseUrl = baseUrl;
    this.sessionId = null;
    this.chatId = null;
  }

  async sendMessage(chatbotId, query) {
    const response = await fetch(`${this.baseUrl}/chat`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        chatbot_id: chatbotId,
        query: query,
        session_id: this.sessionId,
        chat_id: this.chatId
      })
    });

    const data = await response.json();
    
    if (data.success) {
      this.sessionId = data.data.session_id;
      this.chatId = data.data.chat_id;
      return data.data;
    } else {
      throw new Error(data.message);
    }
  }

  async getChatHistory() {
    if (!this.chatId) return [];
    
    const response = await fetch(`${this.baseUrl}/chat/history?chat_id=${this.chatId}`);
    const data = await response.json();
    
    return data.success ? data.data.conversations : [];
  }
}
```

### 2. Streaming Chat Implementation

```javascript
class StreamingRAGChatClient {
  constructor(baseUrl = 'http://localhost:8000/api') {
    this.baseUrl = baseUrl;
    this.sessionId = null;
    this.chatId = null;
  }

  async startStreamingChat(chatbotId, query, onChunk, onComplete, onError) {
    const requestBody = {
      chatbot_id: chatbotId,
      query: query,
      session_id: this.sessionId,
      chat_id: this.chatId
    };

    // Note: EventSource doesn't support POST requests directly
    // You'll need to use fetch with ReadableStream or a library like EventSource polyfill
    
    const response = await fetch(`${this.baseUrl}/chat/stream`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(requestBody)
    });

    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }

    const reader = response.body.getReader();
    const decoder = new TextDecoder();

    try {
      while (true) {
        const { done, value } = await reader.read();
        
        if (done) break;
        
        const chunk = decoder.decode(value);
        const lines = chunk.split('\n');
        
        for (const line of lines) {
          if (line.startsWith('data: ')) {
            const jsonData = line.slice(6);
            if (jsonData.trim() === '') continue;
            
            try {
              const data = JSON.parse(jsonData);
              onChunk(data);
              
              if (data.is_final) {
                this.sessionId = data.session_id;
                this.chatId = data.chat_id;
                onComplete(data);
                return;
              }
            } catch (e) {
              console.error('Error parsing SSE data:', e);
            }
          }
        }
      }
    } catch (error) {
      onError(error);
    } finally {
      reader.releaseLock();
    }
  }
}
```

### 3. React Hook Example

```javascript
import { useState, useCallback } from 'react';

export function useRAGChat() {
  const [messages, setMessages] = useState([]);
  const [isLoading, setIsLoading] = useState(false);
  const [sessionId, setSessionId] = useState(null);
  const [chatId, setChatId] = useState(null);

  const sendMessage = useCallback(async (chatbotId, query) => {
    setIsLoading(true);
    
    try {
      const response = await fetch('/api/chat', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          chatbot_id: chatbotId,
          query: query,
          session_id: sessionId,
          chat_id: chatId
        })
      });

      const data = await response.json();
      
      if (data.success) {
        setSessionId(data.data.session_id);
        setChatId(data.data.chat_id);
        
        setMessages(prev => [
          ...prev,
          { role: 'user', content: query },
          { role: 'assistant', content: data.data.bot_response }
        ]);
      }
    } catch (error) {
      console.error('Error sending message:', error);
    } finally {
      setIsLoading(false);
    }
  }, [sessionId, chatId]);

  return {
    messages,
    isLoading,
    sendMessage,
    sessionId,
    chatId
  };
}
```

### 4. Vue.js Composable Example

```javascript
import { ref, reactive } from 'vue';

export function useRAGChat() {
  const messages = ref([]);
  const isLoading = ref(false);
  const sessionId = ref(null);
  const chatId = ref(null);

  const sendMessage = async (chatbotId, query) => {
    isLoading.value = true;
    
    try {
      const response = await fetch('/api/chat', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          chatbot_id: chatbotId,
          query: query,
          session_id: sessionId.value,
          chat_id: chatId.value
        })
      });

      const data = await response.json();
      
      if (data.success) {
        sessionId.value = data.data.session_id;
        chatId.value = data.data.chat_id;
        
        messages.value.push(
          { role: 'user', content: query },
          { role: 'assistant', content: data.data.bot_response }
        );
      }
    } catch (error) {
      console.error('Error sending message:', error);
    } finally {
      isLoading.value = false;
    }
  };

  return {
    messages,
    isLoading,
    sendMessage,
    sessionId,
    chatId
  };
}
```

---

## Error Handling

### HTTP Status Codes

- `200`: Success
- `400`: Bad Request (invalid parameters)
- `404`: Not Found (session/chat not found)
- `500`: Internal Server Error

### Error Response Format

```json
{
  "success": false,
  "message": "Error description",
  "error": "Detailed error information"
}
```

### Frontend Error Handling

```javascript
async function handleApiCall(apiCall) {
  try {
    const response = await apiCall();
    
    if (!response.ok) {
      const errorData = await response.json();
      throw new Error(errorData.message || 'API request failed');
    }
    
    return await response.json();
  } catch (error) {
    console.error('API Error:', error);
    // Handle error in UI
    throw error;
  }
}
```

---

## Best Practices

### 1. Session Management
- Store `session_id` and `chat_id` in localStorage or sessionStorage
- Reuse existing sessions for better conversation context
- Create new sessions when starting fresh conversations

### 2. Streaming Implementation
- Use proper error handling for streaming connections
- Implement reconnection logic for dropped connections
- Show loading indicators during streaming
- Handle partial responses gracefully

### 3. Performance Optimization
- Implement request debouncing for rapid user input
- Cache chatbot information
- Use pagination for chat history if needed

### 4. User Experience
- Show typing indicators during streaming
- Display context information (files used)
- Provide clear error messages
- Implement conversation persistence

---

## Testing

### Test Endpoints

**GET** `/chat/test-sse`

Test Server-Sent Events functionality.

**Response:** SSE stream with test data

### Example Test Script

```javascript
// Test regular chat
async function testRegularChat() {
  const response = await fetch('/api/chat', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      chatbot_id: 'your-chatbot-id',
      query: 'Test message'
    })
  });
  
  const data = await response.json();
  console.log('Regular chat response:', data);
}

// Test streaming chat
async function testStreamingChat() {
  const response = await fetch('/api/chat/stream', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      chatbot_id: 'your-chatbot-id',
      query: 'Test streaming message'
    })
  });
  
  const reader = response.body.getReader();
  const decoder = new TextDecoder();
  
  while (true) {
    const { done, value } = await reader.read();
    if (done) break;
    
    const chunk = decoder.decode(value);
    console.log('Streaming chunk:', chunk);
  }
}
```

---

## Support

For technical support or questions about the API, please refer to the backend logs or contact the development team.

**API Version:** 1.0  
**Last Updated:** 2024-01-01  
**Backend Version:** RAG_Rust v0.1.0
