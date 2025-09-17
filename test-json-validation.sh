#!/bin/bash

# Test JSON validation error handling

echo "Testing JSON Validation Error Handling"
echo "======================================="

# Test 1: Malformed JSON (syntax error)
echo -e "\n1. Testing malformed JSON:"
curl -s -X POST http://localhost:3000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"invalid json' | jq . || echo "Response: $(curl -s -X POST http://localhost:3000/api/auth/login -H "Content-Type: application/json" -d '{"invalid json')"

# Test 2: Missing Content-Type header
echo -e "\n2. Testing missing Content-Type header:"
curl -s -X POST http://localhost:3000/api/auth/login \
  -d '{"email":"test@example.com","password":"password"}' | jq . || echo "Response: $(curl -s -X POST http://localhost:3000/api/auth/login -d '{"email":"test@example.com","password":"password"}')"

# Test 3: Empty body
echo -e "\n3. Testing empty body:"
curl -s -X POST http://localhost:3000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '' | jq . || echo "Response: $(curl -s -X POST http://localhost:3000/api/auth/login -H "Content-Type: application/json" -d '')"

# Test 4: Invalid JSON structure (missing required fields)
echo -e "\n4. Testing invalid structure (missing fields):"
curl -s -X POST http://localhost:3000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com"}' | jq . || echo "Response: $(curl -s -X POST http://localhost:3000/api/auth/login -H "Content-Type: application/json" -d '{"email":"test@example.com"}')"

# Test 5: Valid JSON
echo -e "\n5. Testing valid JSON (should fail auth but parse correctly):"
curl -s -X POST http://localhost:3000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"password"}' | jq . || echo "Response: $(curl -s -X POST http://localhost:3000/api/auth/login -H "Content-Type: application/json" -d '{"email":"test@example.com","password":"password"}')"

echo -e "\nDone!"