#!/bin/bash

# Configuration
HOST="0.0.0.0:50053" # Match your gRPC server address

# Color codes for output
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to test streaming endpoint
test_streaming_endpoint() {
  local sentence="$1"
  local description="$2"

  echo -e "${BLUE}Testing: $description${NC}"
  echo "Sentence: $sentence"
  echo "-----------------"

  REQUEST_PAYLOAD=$(
    cat <<EOF
{
    "sentence": "$sentence"
}
EOF
  )

  echo "Request payload:"
  echo "$REQUEST_PAYLOAD"
  echo "-----------------"

  # Call gRPC streaming endpoint
  response=$(grpcurl -plaintext \
    -d "$REQUEST_PAYLOAD" \
    $HOST \
    sentence.SentenceService/AnalyzeSentence 2>&1)

  if [ $? -eq 0 ]; then
    echo -e "${GREEN}Success:${NC}"
    echo "$response"
  else
    echo -e "${RED}Error:${NC}"
    echo "$response"
  fi
  echo "-----------------"
  echo
}

# Test streaming response
test_streaming_endpoint "Analyze this sentence" "Streaming test"

# List available services (for verification)
echo "Checking available services:"
echo "-----------------"
grpcurl -plaintext $HOST list
echo

# Show service description
echo "Service description:"
echo "-----------------"
grpcurl -plaintext $HOST describe sentence.SentenceService
echo

echo "All tests completed."
