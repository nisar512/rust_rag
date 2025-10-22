# RAG Rust Backend API Integration Guide

A comprehensive guide for integrating the RAG (Retrieval-Augmented Generation) Rust backend with React.js frontend applications.

## Table of Contents

- [Overview](#overview)
- [Quick Start](#quick-start)
- [API Endpoints](#api-endpoints)
- [React.js Integration](#reactjs-integration)
- [Error Handling](#error-handling)
- [Best Practices](#best-practices)
- [Troubleshooting](#troubleshooting)

## Overview

The RAG Rust backend provides a powerful API for building AI-powered chat applications with document knowledge bases. It supports:

- **Document Upload & Processing**: Upload PDF files and automatically create embeddings
- **Chatbot Management**: Create and manage multiple chatbots
- **Real-time Chat**: Both regular and streaming chat capabilities
- **Vector Search**: Semantic search across uploaded documents
- **Session Management**: Persistent conversation history

## Quick Start

### Prerequisites

- Node.js 16+ and npm/yarn
- Rust backend running on `http://localhost:8000`
- Elasticsearch running on `http://localhost:9200`

### Backend Setup

1. **Start Elasticsearch**:
   ```bash
   docker run -p 9200:9200 -e 'discovery.type=single-node' elasticsearch:8.15.0
   ```

2. **Start the Rust backend**:
   ```bash
   cd /path/to/rag-rust-backend
   cargo run
   ```

3. **Set environment variables**:
   ```bash
   export GEMINI_API_KEY="your_gemini_api_key_here"
   export ELASTICSEARCH_URL="http://localhost:9200"
   ```

### Frontend Setup

1. **Install dependencies**:
   ```bash
   npm install axios react-query
   # or
   yarn add axios react-query
   ```

2. **Create API client**:
   ```javascript
   // src/api/client.js
   import axios from 'axios';

   const API_BASE_URL = 'http://localhost:8000/api';

   export const apiClient = axios.create({
     baseURL: API_BASE_URL,
     headers: {
       'Content-Type': 'application/json',
     },
   });

   // Add request interceptor for error handling
   apiClient.interceptors.response.use(
     (response) => response,
     (error) => {
       console.error('API Error:', error.response?.data || error.message);
       return Promise.reject(error);
     }
   );
   ```

## API Endpoints

### Base URL
```
http://localhost:8000/api
```

### 1. Health Check

**GET** `/health`

Check if the server is running.

```javascript
const checkHealth = async () => {
  const response = await apiClient.get('/health');
  return response.data;
};
```

**Response:**
```json
{
  "status": "ok",
  "message": "RAG Server is running",
  "timestamp": "2024-01-01T00:00:00Z"
}
```

### 2. Chatbot Management

#### Create Chatbot

**POST** `/chatbots`

Create a new chatbot.

```javascript
const createChatbot = async (name) => {
  const response = await apiClient.post('/chatbots', { name });
  return response.data;
};
```

**Request Body:**
```json
{
  "name": "My Knowledge Bot"
}
```

**Response:**
```json
{
  "success": true,
  "message": "Chatbot created successfully",
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "My Knowledge Bot",
    "created_at": "2024-01-01T00:00:00Z",
    "updated_at": "2024-01-01T00:00:00Z",
    "status": "active"
  }
}
```

#### Get All Chatbots

**GET** `/chatbots`

Retrieve all available chatbots.

```javascript
const getChatbots = async () => {
  const response = await apiClient.get('/chatbots');
  return response.data;
};
```

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

### 3. Document Upload

#### Upload PDF

**POST** `/upload-pdf`

Upload a PDF file and create embeddings for a chatbot.

```javascript
const uploadPDF = async (chatbotId, file) => {
  const formData = new FormData();
  formData.append('chatbot_id', chatbotId);
  formData.append('file', file);

  const response = await apiClient.post('/upload-pdf', formData, {
    headers: {
      'Content-Type': 'multipart/form-data',
    },
  });
  return response.data;
};
```

**Request:** Multipart form data
- `chatbot_id`: UUID string (required)
- `file`: PDF file (required)

**Response:**
```json
{
  "success": true,
  "message": "PDF uploaded and processed successfully",
  "data": {
    "chatbot_id": "550e8400-e29b-41d4-a716-446655440000",
    "file_name": "document.pdf",
    "chunks_created": 15,
    "processing_time": "2.5s"
  }
}
```

### 4. Chat Endpoints

#### Regular Chat

**POST** `/chat`

Send a message to the chatbot and receive a complete response.

```javascript
const sendMessage = async (chatbotId, query, sessionId = null, chatId = null) => {
  const response = await apiClient.post('/chat', {
    chatbot_id: chatbotId,
    query: query,
    session_id: sessionId,
    chat_id: chatId
  });
  return response.data;
};
```

**Request Body:**
```json
{
  "chatbot_id": "550e8400-e29b-41d4-a716-446655440000",
  "query": "What is this document about?",
  "session_id": "optional-session-id",
  "chat_id": "optional-chat-id"
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
    "user_query": "What is this document about?",
    "bot_response": "This document discusses...",
    "context_used": ["document.pdf"]
  }
}
```

#### Streaming Chat

**POST** `/chat/stream`

Send a message to the chatbot and receive streaming responses via Server-Sent Events (SSE).

```javascript
const startStreamingChat = async (chatbotId, query, onChunk, onComplete, onError) => {
  const requestBody = {
    chatbot_id: chatbotId,
    query: query,
    session_id: sessionId,
    chat_id: chatId
  };

  const response = await fetch(`${API_BASE_URL}/chat/stream`, {
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
};
```

#### Create Session

**POST** `/chat/session`

Create a new chat session.

```javascript
const createSession = async () => {
  const response = await apiClient.post('/chat/session');
  return response.data;
};
```

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

#### Get Chat History

**GET** `/chat/history?chat_id={chat_id}`

Retrieve conversation history for a specific chat.

```javascript
const getChatHistory = async (chatId) => {
  const response = await apiClient.get(`/chat/history?chat_id=${chatId}`);
  return response.data;
};
```

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
        "user_query": "What is this document about?",
        "bot_response": "This document discusses...",
        "created_at": "2024-01-01T00:00:00Z"
      }
    ],
    "count": 1
  }
}
```

### 5. Query Endpoints

#### Semantic Search

**GET** `/query?chatbot_id={id}&query={query}&limit={limit}`

Perform semantic search across uploaded documents.

```javascript
const searchDocuments = async (chatbotId, query, limit = 5) => {
  const response = await apiClient.get('/query', {
    params: {
      chatbot_id: chatbotId,
      query: query,
      limit: limit
    }
  });
  return response.data;
};
```

**Response:**
```json
{
  "success": true,
  "message": "Query processed successfully",
  "data": {
    "chatbot_id": "550e8400-e29b-41d4-a716-446655440000",
    "query": "machine learning",
    "results": [
      {
        "id": "doc_chunk_1",
        "content": "Machine learning is a subset of artificial intelligence...",
        "score": 0.95,
        "metadata": {
          "file_name": "ml_guide.pdf",
          "page_number": 1
        }
      }
    ],
    "total_results": 1
  }
}
```

## React.js Integration

### 1. Custom Hooks

#### useRAGChat Hook

```javascript
// src/hooks/useRAGChat.js
import { useState, useCallback } from 'react';
import { apiClient } from '../api/client';

export const useRAGChat = () => {
  const [messages, setMessages] = useState([]);
  const [isLoading, setIsLoading] = useState(false);
  const [sessionId, setSessionId] = useState(null);
  const [chatId, setChatId] = useState(null);
  const [error, setError] = useState(null);

  const sendMessage = useCallback(async (chatbotId, query) => {
    setIsLoading(true);
    setError(null);
    
    try {
      const response = await apiClient.post('/chat', {
        chatbot_id: chatbotId,
        query: query,
        session_id: sessionId,
        chat_id: chatId
      });

      if (response.data.success) {
        setSessionId(response.data.data.session_id);
        setChatId(response.data.data.chat_id);
        
        setMessages(prev => [
          ...prev,
          { role: 'user', content: query },
          { 
            role: 'assistant', 
            content: response.data.data.bot_response,
            context: response.data.data.context_used
          }
        ]);
      }
    } catch (error) {
      setError(error.response?.data?.message || 'Failed to send message');
    } finally {
      setIsLoading(false);
    }
  }, [sessionId, chatId]);

  const getChatHistory = useCallback(async () => {
    if (!chatId) return [];
    
    try {
      const response = await apiClient.get(`/chat/history?chat_id=${chatId}`);
      return response.data.success ? response.data.data.conversations : [];
    } catch (error) {
      setError(error.response?.data?.message || 'Failed to get chat history');
      return [];
    }
  }, [chatId]);

  return {
    messages,
    isLoading,
    error,
    sendMessage,
    getChatHistory,
    sessionId,
    chatId
  };
};
```

#### useStreamingChat Hook

```javascript
// src/hooks/useStreamingChat.js
import { useState, useCallback } from 'react';

export const useStreamingChat = () => {
  const [messages, setMessages] = useState([]);
  const [isStreaming, setIsStreaming] = useState(false);
  const [currentResponse, setCurrentResponse] = useState('');
  const [sessionId, setSessionId] = useState(null);
  const [chatId, setChatId] = useState(null);
  const [error, setError] = useState(null);

  const startStreamingChat = useCallback(async (chatbotId, query) => {
    setIsStreaming(true);
    setError(null);
    setCurrentResponse('');
    
    // Add user message immediately
    setMessages(prev => [...prev, { role: 'user', content: query }]);

    try {
      const response = await fetch(`${process.env.REACT_APP_API_URL || 'http://localhost:8000/api'}/chat/stream`, {
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

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }

      const reader = response.body.getReader();
      const decoder = new TextDecoder();

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
              setCurrentResponse(prev => prev + data.text);
              
              if (data.is_final) {
                setSessionId(data.session_id);
                setChatId(data.chat_id);
                setMessages(prev => [
                  ...prev,
                  { 
                    role: 'assistant', 
                    content: prev[prev.length - 1].content + data.text,
                    context: data.context_used
                  }
                ]);
                setCurrentResponse('');
                break;
              }
            } catch (e) {
              console.error('Error parsing SSE data:', e);
            }
          }
        }
      }
    } catch (error) {
      setError(error.message);
    } finally {
      setIsStreaming(false);
      reader.releaseLock();
    }
  }, [sessionId, chatId]);

  return {
    messages,
    isStreaming,
    currentResponse,
    error,
    startStreamingChat,
    sessionId,
    chatId
  };
};
```

#### useChatbots Hook

```javascript
// src/hooks/useChatbots.js
import { useState, useEffect, useCallback } from 'react';
import { apiClient } from '../api/client';

export const useChatbots = () => {
  const [chatbots, setChatbots] = useState([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState(null);

  const fetchChatbots = useCallback(async () => {
    setIsLoading(true);
    setError(null);
    
    try {
      const response = await apiClient.get('/chatbots');
      if (response.data.success) {
        setChatbots(response.data.data);
      }
    } catch (error) {
      setError(error.response?.data?.message || 'Failed to fetch chatbots');
    } finally {
      setIsLoading(false);
    }
  }, []);

  const createChatbot = useCallback(async (name) => {
    try {
      const response = await apiClient.post('/chatbots', { name });
      if (response.data.success) {
        setChatbots(prev => [...prev, response.data.data]);
        return response.data.data;
      }
    } catch (error) {
      setError(error.response?.data?.message || 'Failed to create chatbot');
      throw error;
    }
  }, []);

  useEffect(() => {
    fetchChatbots();
  }, [fetchChatbots]);

  return {
    chatbots,
    isLoading,
    error,
    fetchChatbots,
    createChatbot
  };
};
```

### 2. React Components

#### Chat Component

```javascript
// src/components/Chat.jsx
import React, { useState, useRef, useEffect } from 'react';
import { useRAGChat } from '../hooks/useRAGChat';

const Chat = ({ chatbotId }) => {
  const { messages, isLoading, error, sendMessage } = useRAGChat();
  const [input, setInput] = useState('');
  const messagesEndRef = useRef(null);

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  };

  useEffect(() => {
    scrollToBottom();
  }, [messages]);

  const handleSubmit = async (e) => {
    e.preventDefault();
    if (!input.trim() || isLoading) return;

    await sendMessage(chatbotId, input);
    setInput('');
  };

  return (
    <div className="chat-container">
      <div className="messages">
        {messages.map((message, index) => (
          <div key={index} className={`message ${message.role}`}>
            <div className="content">{message.content}</div>
            {message.context && (
              <div className="context">
                Sources: {message.context.join(', ')}
              </div>
            )}
          </div>
        ))}
        {isLoading && (
          <div className="message assistant">
            <div className="typing-indicator">...</div>
          </div>
        )}
        <div ref={messagesEndRef} />
      </div>
      
      {error && <div className="error">{error}</div>}
      
      <form onSubmit={handleSubmit} className="input-form">
        <input
          type="text"
          value={input}
          onChange={(e) => setInput(e.target.value)}
          placeholder="Type your message..."
          disabled={isLoading}
        />
        <button type="submit" disabled={isLoading || !input.trim()}>
          Send
        </button>
      </form>
    </div>
  );
};

export default Chat;
```

#### Streaming Chat Component

```javascript
// src/components/StreamingChat.jsx
import React, { useState, useRef, useEffect } from 'react';
import { useStreamingChat } from '../hooks/useStreamingChat';

const StreamingChat = ({ chatbotId }) => {
  const { 
    messages, 
    isStreaming, 
    currentResponse, 
    error, 
    startStreamingChat 
  } = useStreamingChat();
  
  const [input, setInput] = useState('');
  const messagesEndRef = useRef(null);

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  };

  useEffect(() => {
    scrollToBottom();
  }, [messages, currentResponse]);

  const handleSubmit = async (e) => {
    e.preventDefault();
    if (!input.trim() || isStreaming) return;

    await startStreamingChat(chatbotId, input);
    setInput('');
  };

  return (
    <div className="chat-container">
      <div className="messages">
        {messages.map((message, index) => (
          <div key={index} className={`message ${message.role}`}>
            <div className="content">{message.content}</div>
            {message.context && (
              <div className="context">
                Sources: {message.context.join(', ')}
              </div>
            )}
          </div>
        ))}
        
        {isStreaming && currentResponse && (
          <div className="message assistant">
            <div className="content">
              {currentResponse}
              <span className="cursor">|</span>
            </div>
          </div>
        )}
        
        <div ref={messagesEndRef} />
      </div>
      
      {error && <div className="error">{error}</div>}
      
      <form onSubmit={handleSubmit} className="input-form">
        <input
          type="text"
          value={input}
          onChange={(e) => setInput(e.target.value)}
          placeholder="Type your message..."
          disabled={isStreaming}
        />
        <button type="submit" disabled={isStreaming || !input.trim()}>
          {isStreaming ? 'Sending...' : 'Send'}
        </button>
      </form>
    </div>
  );
};

export default StreamingChat;
```

#### File Upload Component

```javascript
// src/components/FileUpload.jsx
import React, { useState } from 'react';
import { apiClient } from '../api/client';

const FileUpload = ({ chatbotId, onUploadComplete }) => {
  const [file, setFile] = useState(null);
  const [isUploading, setIsUploading] = useState(false);
  const [error, setError] = useState(null);
  const [progress, setProgress] = useState(0);

  const handleFileChange = (e) => {
    const selectedFile = e.target.files[0];
    if (selectedFile && selectedFile.type === 'application/pdf') {
      setFile(selectedFile);
      setError(null);
    } else {
      setError('Please select a PDF file');
    }
  };

  const handleUpload = async () => {
    if (!file || !chatbotId) return;

    setIsUploading(true);
    setError(null);
    setProgress(0);

    try {
      const formData = new FormData();
      formData.append('chatbot_id', chatbotId);
      formData.append('file', file);

      const response = await apiClient.post('/upload-pdf', formData, {
        headers: {
          'Content-Type': 'multipart/form-data',
        },
        onUploadProgress: (progressEvent) => {
          const percentCompleted = Math.round(
            (progressEvent.loaded * 100) / progressEvent.total
          );
          setProgress(percentCompleted);
        },
      });

      if (response.data.success) {
        onUploadComplete?.(response.data.data);
        setFile(null);
        setProgress(0);
      }
    } catch (error) {
      setError(error.response?.data?.message || 'Upload failed');
    } finally {
      setIsUploading(false);
    }
  };

  return (
    <div className="file-upload">
      <div className="upload-area">
        <input
          type="file"
          accept=".pdf"
          onChange={handleFileChange}
          disabled={isUploading}
        />
        {file && (
          <div className="file-info">
            <p>Selected: {file.name}</p>
            <p>Size: {(file.size / 1024 / 1024).toFixed(2)} MB</p>
          </div>
        )}
      </div>

      {isUploading && (
        <div className="progress">
          <div 
            className="progress-bar" 
            style={{ width: `${progress}%` }}
          />
          <span>{progress}%</span>
        </div>
      )}

      {error && <div className="error">{error}</div>}

      <button 
        onClick={handleUpload}
        disabled={!file || isUploading}
        className="upload-button"
      >
        {isUploading ? 'Uploading...' : 'Upload PDF'}
      </button>
    </div>
  );
};

export default FileUpload;
```

### 3. Context Provider

```javascript
// src/context/RAGContext.jsx
import React, { createContext, useContext, useState } from 'react';

const RAGContext = createContext();

export const RAGProvider = ({ children }) => {
  const [selectedChatbot, setSelectedChatbot] = useState(null);
  const [sessionId, setSessionId] = useState(null);
  const [chatId, setChatId] = useState(null);

  const value = {
    selectedChatbot,
    setSelectedChatbot,
    sessionId,
    setSessionId,
    chatId,
    setChatId,
  };

  return (
    <RAGContext.Provider value={value}>
      {children}
    </RAGContext.Provider>
  );
};

export const useRAGContext = () => {
  const context = useContext(RAGContext);
  if (!context) {
    throw new Error('useRAGContext must be used within RAGProvider');
  }
  return context;
};
```

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

### React Error Boundary

```javascript
// src/components/ErrorBoundary.jsx
import React from 'react';

class ErrorBoundary extends React.Component {
  constructor(props) {
    super(props);
    this.state = { hasError: false, error: null };
  }

  static getDerivedStateFromError(error) {
    return { hasError: true, error };
  }

  componentDidCatch(error, errorInfo) {
    console.error('Error caught by boundary:', error, errorInfo);
  }

  render() {
    if (this.state.hasError) {
      return (
        <div className="error-boundary">
          <h2>Something went wrong</h2>
          <p>{this.state.error?.message}</p>
          <button onClick={() => this.setState({ hasError: false, error: null })}>
            Try again
          </button>
        </div>
      );
    }

    return this.props.children;
  }
}

export default ErrorBoundary;
```

### API Error Handler

```javascript
// src/utils/errorHandler.js
export const handleApiError = (error) => {
  if (error.response) {
    // Server responded with error status
    const { status, data } = error.response;
    
    switch (status) {
      case 400:
        return `Bad Request: ${data.message || 'Invalid parameters'}`;
      case 404:
        return `Not Found: ${data.message || 'Resource not found'}`;
      case 500:
        return `Server Error: ${data.message || 'Internal server error'}`;
      default:
        return `Error ${status}: ${data.message || 'Unknown error'}`;
    }
  } else if (error.request) {
    // Request was made but no response received
    return 'Network Error: Unable to connect to server';
  } else {
    // Something else happened
    return `Error: ${error.message}`;
  }
};
```

## Best Practices

### 1. State Management

- Use React Query or SWR for server state management
- Implement proper loading and error states
- Cache chatbot data and chat history
- Use localStorage for session persistence

### 2. Performance Optimization

- Implement request debouncing for rapid user input
- Use React.memo for expensive components
- Implement virtual scrolling for long chat histories
- Lazy load components when possible

### 3. User Experience

- Show typing indicators during streaming
- Display upload progress for file uploads
- Implement retry mechanisms for failed requests
- Provide clear error messages and recovery options

### 4. Security

- Validate file types and sizes on the frontend
- Implement proper error handling to avoid exposing sensitive information
- Use HTTPS in production
- Sanitize user inputs before sending to API

### 5. Code Organization

```javascript
// src/
├── api/
│   ├── client.js
│   ├── endpoints.js
│   └── types.js
├── components/
│   ├── Chat.jsx
│   ├── StreamingChat.jsx
│   ├── FileUpload.jsx
│   └── ErrorBoundary.jsx
├── hooks/
│   ├── useRAGChat.js
│   ├── useStreamingChat.js
│   └── useChatbots.js
├── context/
│   └── RAGContext.jsx
├── utils/
│   ├── errorHandler.js
│   └── constants.js
└── styles/
    └── components.css
```

## Troubleshooting

### Common Issues

1. **CORS Errors**
   - Ensure the backend CORS configuration allows your frontend origin
   - Check that preflight requests are handled correctly

2. **File Upload Failures**
   - Verify file size limits (default: 10MB)
   - Ensure file is PDF format
   - Check chatbot ID is valid UUID

3. **Streaming Connection Issues**
   - Verify EventSource support in browser
   - Check network connectivity
   - Implement reconnection logic

4. **Authentication Issues**
   - Ensure GEMINI_API_KEY is set in backend
   - Check API key validity and permissions

### Debug Mode

Enable debug logging in your React app:

```javascript
// src/utils/logger.js
export const logger = {
  debug: (message, data) => {
    if (process.env.NODE_ENV === 'development') {
      console.log(`[DEBUG] ${message}`, data);
    }
  },
  error: (message, error) => {
    console.error(`[ERROR] ${message}`, error);
  }
};
```

### Testing

```javascript
// src/utils/testHelpers.js
export const mockApiResponse = (data, success = true) => ({
  data: {
    success,
    message: success ? 'Success' : 'Error',
    data: success ? data : null,
    error: success ? null : data
  }
});

export const createMockChatbot = (id = 'test-id') => ({
  id,
  name: 'Test Chatbot',
  created_at: new Date().toISOString(),
  updated_at: new Date().toISOString(),
  status: 'active'
});
```

## Environment Variables

Create a `.env` file in your React project:

```bash
REACT_APP_API_URL=http://localhost:8000/api
REACT_APP_DEBUG=true
```

## Support

For technical support or questions about the API:

- Check the backend logs for detailed error information
- Verify all environment variables are set correctly
- Ensure Elasticsearch and the backend are running
- Test API endpoints directly using curl or Postman

**API Version:** 1.0  
**Last Updated:** 2024-01-01  
**Backend Version:** RAG_Rust v0.1.0
