# Quick Frontend Integration Guide

## 🚀 Quick Start

### Base URL
```
http://localhost:8000/api
```

### Essential Endpoints

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/chatbots` | GET | Get all chatbots |
| `/chat` | POST | Send message (regular) |
| `/chat/stream` | POST | Send message (streaming) |
| `/chat/session` | POST | Create new session |
| `/chat/history` | GET | Get chat history |

---

## 📝 Basic Integration

### 1. Get Available Chatbots

```javascript
const chatbots = await fetch('/api/chatbots').then(r => r.json());
console.log(chatbots.data); // Array of chatbots
```

### 2. Send Regular Chat Message

```javascript
const response = await fetch('/api/chat', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    chatbot_id: 'your-chatbot-id',
    query: 'Hello, what can you help me with?'
  })
});

const data = await response.json();
console.log(data.data.bot_response); // Bot's response
```

### 3. Send Streaming Chat Message

```javascript
async function streamChat(chatbotId, query) {
  const response = await fetch('/api/chat/stream', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      chatbot_id: chatbotId,
      query: query
    })
  });

  const reader = response.body.getReader();
  const decoder = new TextDecoder();
  
  while (true) {
    const { done, value } = await reader.read();
    if (done) break;
    
    const chunk = decoder.decode(value);
    const lines = chunk.split('\n');
    
    for (const line of lines) {
      if (line.startsWith('data: ')) {
        const data = JSON.parse(line.slice(6));
        console.log('Chunk:', data.text);
        
        if (data.is_final) {
          console.log('Streaming complete!');
          break;
        }
      }
    }
  }
}
```

---

## 🎯 Request/Response Examples

### Regular Chat Request
```json
{
  "chatbot_id": "b43c1d87-789e-46d2-aa0a-de1a5a8a1092",
  "query": "What brands are mentioned in the document?",
  "session_id": "optional-session-id",
  "chat_id": "optional-chat-id"
}
```

### Regular Chat Response
```json
{
  "success": true,
  "message": "Chat request processed successfully",
  "data": {
    "session_id": "550e8400-e29b-41d4-a716-446655440000",
    "chat_id": "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
    "conversation_id": "6ba7b811-9dad-11d1-80b4-00c04fd430c8",
    "user_query": "What brands are mentioned in the document?",
    "bot_response": "The following brands are mentioned: Frolic, Wolfsblut, ANIﬁt, Fidelis, Wynn",
    "context_used": ["document.pdf"]
  }
}
```

### Streaming Chat Response (SSE)
```
data: {"text": "The following", "is_final": false, "session_id": "uuid", "chat_id": "uuid"}
data: {"text": "brands are mentioned:", "is_final": false, "session_id": "uuid", "chat_id": "uuid"}
data: {"text": "Frolic, Wolfsblut, ANIﬁt", "is_final": true, "session_id": "uuid", "chat_id": "uuid"}
```

---

## 🔧 React Example

```jsx
import React, { useState, useEffect } from 'react';

function ChatComponent() {
  const [messages, setMessages] = useState([]);
  const [input, setInput] = useState('');
  const [chatbotId, setChatbotId] = useState('');
  const [sessionId, setSessionId] = useState(null);
  const [chatId, setChatId] = useState(null);

  // Get chatbots on component mount
  useEffect(() => {
    fetch('/api/chatbots')
      .then(r => r.json())
      .then(data => {
        if (data.success && data.data.length > 0) {
          setChatbotId(data.data[0].id);
        }
      });
  }, []);

  const sendMessage = async () => {
    if (!input.trim() || !chatbotId) return;

    const response = await fetch('/api/chat', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        chatbot_id: chatbotId,
        query: input,
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
        { role: 'user', content: input },
        { role: 'assistant', content: data.data.bot_response }
      ]);
      
      setInput('');
    }
  };

  return (
    <div>
      <div className="messages">
        {messages.map((msg, i) => (
          <div key={i} className={`message ${msg.role}`}>
            {msg.content}
          </div>
        ))}
      </div>
      
      <div className="input">
        <input
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyPress={(e) => e.key === 'Enter' && sendMessage()}
          placeholder="Type your message..."
        />
        <button onClick={sendMessage}>Send</button>
      </div>
    </div>
  );
}
```

---

## 🎨 Vue.js Example

```vue
<template>
  <div class="chat-container">
    <div class="messages">
      <div 
        v-for="(message, index) in messages" 
        :key="index"
        :class="['message', message.role]"
      >
        {{ message.content }}
      </div>
    </div>
    
    <div class="input-section">
      <input
        v-model="input"
        @keyup.enter="sendMessage"
        placeholder="Type your message..."
      />
      <button @click="sendMessage" :disabled="!input.trim()">
        Send
      </button>
    </div>
  </div>
</template>

<script>
export default {
  data() {
    return {
      messages: [],
      input: '',
      chatbotId: '',
      sessionId: null,
      chatId: null
    };
  },
  
  async mounted() {
    const response = await fetch('/api/chatbots');
    const data = await response.json();
    
    if (data.success && data.data.length > 0) {
      this.chatbotId = data.data[0].id;
    }
  },
  
  methods: {
    async sendMessage() {
      if (!this.input.trim() || !this.chatbotId) return;

      const response = await fetch('/api/chat', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          chatbot_id: this.chatbotId,
          query: this.input,
          session_id: this.sessionId,
          chat_id: this.chatId
        })
      });

      const data = await response.json();
      
      if (data.success) {
        this.sessionId = data.data.session_id;
        this.chatId = data.data.chat_id;
        
        this.messages.push(
          { role: 'user', content: this.input },
          { role: 'assistant', content: data.data.bot_response }
        );
        
        this.input = '';
      }
    }
  }
};
</script>
```

---

## ⚡ Streaming Example

```javascript
class StreamingChat {
  constructor(chatbotId) {
    this.chatbotId = chatbotId;
    this.sessionId = null;
    this.chatId = null;
  }

  async sendStreamingMessage(query, onChunk, onComplete) {
    const response = await fetch('/api/chat/stream', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        chatbot_id: this.chatbotId,
        query: query,
        session_id: this.sessionId,
        chat_id: this.chatId
      })
    });

    const reader = response.body.getReader();
    const decoder = new TextDecoder();
    let fullResponse = '';

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
              fullResponse += data.text;
              onChunk(data.text);

              if (data.is_final) {
                this.sessionId = data.session_id;
                this.chatId = data.chat_id;
                onComplete(fullResponse);
                return;
              }
            } catch (e) {
              console.error('Error parsing SSE data:', e);
            }
          }
        }
      }
    } finally {
      reader.releaseLock();
    }
  }
}

// Usage
const chat = new StreamingChat('your-chatbot-id');

chat.sendStreamingMessage(
  'Tell me about the document',
  (chunk) => console.log('Received:', chunk),
  (fullResponse) => console.log('Complete:', fullResponse)
);
```

---

## 🚨 Error Handling

```javascript
async function safeApiCall(url, options) {
  try {
    const response = await fetch(url, options);
    
    if (!response.ok) {
      throw new Error(`HTTP ${response.status}: ${response.statusText}`);
    }
    
    return await response.json();
  } catch (error) {
    console.error('API Error:', error);
    // Handle error in your UI
    throw error;
  }
}

// Usage
try {
  const data = await safeApiCall('/api/chat', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ chatbot_id: 'id', query: 'test' })
  });
  
  console.log('Success:', data);
} catch (error) {
  console.error('Failed:', error.message);
}
```

---

## 📋 Checklist for Integration

- [ ] Get chatbot ID from `/api/chatbots`
- [ ] Implement regular chat with `/api/chat`
- [ ] Add session/chat ID persistence
- [ ] Implement streaming chat with `/api/chat/stream`
- [ ] Add proper error handling
- [ ] Test with your chatbot ID
- [ ] Add loading states
- [ ] Implement conversation history

---

## 🔗 Useful Links

- **Full API Documentation**: `FRONTEND_API_DOCS.md`
- **Backend Health**: `http://localhost:8000/health`
- **Chat Health**: `http://localhost:8000/api/chat/health`

---

**Need Help?** Check the backend logs or refer to the full API documentation for detailed examples and error codes.
