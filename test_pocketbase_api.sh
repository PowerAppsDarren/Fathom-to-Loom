#!/bin/bash

# Test script for PocketBase API endpoints
# This assumes the backend server is running on localhost:3000

BASE_URL="http://localhost:3000"
USER_ID="test_user_123"

echo "=== Testing PocketBase Lifecycle Manager API ==="
echo

# Test 1: Health check
echo "1. Testing PocketBase health endpoint..."
curl -s "${BASE_URL}/health/pb" | jq '.'
echo

# Test 2: Initialize PocketBase for user
echo "2. Initializing PocketBase instance for user: ${USER_ID}..."
curl -s -X POST "${BASE_URL}/api/users/${USER_ID}/init_pb" \
  -H "Content-Type: application/json" \
  -d '{"force_restart": false}' | jq '.'
echo

# Test 3: Check user PocketBase status
echo "3. Checking PocketBase status for user: ${USER_ID}..."
curl -s "${BASE_URL}/api/users/${USER_ID}/pb_status" | jq '.'
echo

# Test 4: List all instances
echo "4. Listing all PocketBase instances..."
curl -s "${BASE_URL}/api/pb_instances" | jq '.'
echo

# Test 5: Stop user instance
echo "5. Stopping PocketBase instance for user: ${USER_ID}..."
curl -s -X POST "${BASE_URL}/api/users/${USER_ID}/stop_pb" | jq '.'
echo

echo "=== Test completed ==="
