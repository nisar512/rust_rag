#!/usr/bin/env python3
"""
Test script for the RAG Rust chat endpoints
This script demonstrates how to use the new chat functionality
"""

import requests
import json
import uuid
import time

# Configuration
BASE_URL = "http://localhost:8000/api"
CHATBOT_ID = "your_chatbot_id_here"  # Replace with actual chatbot ID

def test_create_session():
    """Test creating a new session"""
    print("🔄 Testing session creation...")
    
    response = requests.post(f"{BASE_URL}/chat/session")
    
    if response.status_code == 200:
        data = response.json()
        session_id = data["data"]["session_id"]
        print(f"✅ Session created: {session_id}")
        return session_id
    else:
        print(f"❌ Failed to create session: {response.status_code}")
        print(response.text)
        return None

def test_chat_without_session(chatbot_id):
    """Test chat without providing session_id (should create new session)"""
    print("🔄 Testing chat without session_id...")
    
    payload = {
        "chatbot_id": chatbot_id,
        "query": "What is this document about?"
    }
    
    response = requests.post(f"{BASE_URL}/chat", json=payload)
    
    if response.status_code == 200:
        data = response.json()
        print(f"✅ Chat successful!")
        print(f"Session ID: {data['data']['session_id']}")
        print(f"Chat ID: {data['data']['chat_id']}")
        print(f"Bot Response: {data['data']['bot_response']}")
        return data['data']['session_id'], data['data']['chat_id']
    else:
        print(f"❌ Chat failed: {response.status_code}")
        print(response.text)
        return None, None

def test_chat_with_session(chatbot_id, session_id, chat_id):
    """Test chat with existing session and chat"""
    print("🔄 Testing chat with existing session...")
    
    payload = {
        "chatbot_id": chatbot_id,
        "query": "Can you tell me more about the main topics?",
        "session_id": session_id,
        "chat_id": chat_id
    }
    
    response = requests.post(f"{BASE_URL}/chat", json=payload)
    
    if response.status_code == 200:
        data = response.json()
        print(f"✅ Chat successful!")
        print(f"Bot Response: {data['data']['bot_response']}")
        return data['data']['chat_id']
    else:
        print(f"❌ Chat failed: {response.status_code}")
        print(response.text)
        return None

def test_get_chat_history(chat_id):
    """Test getting chat history"""
    print("🔄 Testing chat history retrieval...")
    
    response = requests.get(f"{BASE_URL}/chat/history?chat_id={chat_id}")
    
    if response.status_code == 200:
        data = response.json()
        print(f"✅ Chat history retrieved!")
        print(f"Number of conversations: {data['data']['count']}")
        for conv in data['data']['conversations']:
            print(f"  - User: {conv['user_query']}")
            print(f"    Bot: {conv['bot_response']}")
            print()
    else:
        print(f"❌ Failed to get chat history: {response.status_code}")
        print(response.text)

def test_health_endpoints():
    """Test health endpoints"""
    print("🔄 Testing health endpoints...")
    
    # Test main health
    response = requests.get("http://localhost:8000/health")
    if response.status_code == 200:
        print("✅ Main health check passed")
    else:
        print(f"❌ Main health check failed: {response.status_code}")
    
    # Test chat health
    response = requests.get(f"{BASE_URL}/chat/health")
    if response.status_code == 200:
        print("✅ Chat health check passed")
    else:
        print(f"❌ Chat health check failed: {response.status_code}")

def main():
    """Main test function"""
    print("🚀 Starting RAG Rust Chat API Tests")
    print("=" * 50)
    
    # Test health endpoints first
    test_health_endpoints()
    print()
    
    # Test session creation
    session_id = test_create_session()
    print()
    
    if session_id:
        # Test chat without session (should create new session)
        session_id2, chat_id = test_chat_without_session(CHATBOT_ID)
        print()
        
        if session_id2 and chat_id:
            # Test chat with existing session
            test_chat_with_session(CHATBOT_ID, session_id2, chat_id)
            print()
            
            # Test getting chat history
            test_get_chat_history(chat_id)
    
    print("=" * 50)
    print("🏁 Tests completed!")

if __name__ == "__main__":
    main()
