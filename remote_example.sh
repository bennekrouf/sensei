#!/bin/bash
# remote_example.sh - Example of using APICheck with a remote endpoint service

# Text colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}APICheck with Remote Endpoint Service Examples${NC}"
echo

# Example remote API URL - replace with your actual endpoint service URL
REMOTE_API="http://localhost:50055"
EMAIL="user@example.com"

echo -e "${YELLOW}1. Using CLI with remote endpoints:${NC}"
echo -e "   ${GREEN}cargo run -- --provider claude --api $REMOTE_API --email $EMAIL \"schedule a meeting tomorrow at 2pm with John\"${NC}"
echo

echo -e "${YELLOW}2. Starting gRPC server with remote endpoints:${NC}"
echo -e "   ${GREEN}cargo run -- --provider claude --api $REMOTE_API${NC}"
echo

echo -e "${YELLOW}3. Using CLI with local endpoints (fallback):${NC}"
echo -e "   ${GREEN}cargo run -- --provider claude \"schedule a meeting tomorrow at 2pm with John\"${NC}"
echo

echo -e "${YELLOW}4. Testing connection to remote endpoint service:${NC}"
echo -e "   ${GREEN}grpcurl -plaintext $REMOTE_API list endpoint.EndpointService${NC}"
echo

echo -e "${BLUE}Note:${NC} You need to have an endpoint service running at $REMOTE_API"
echo -e "that implements the endpoint.EndpointService gRPC service."
echo
echo -e "Make sure you have your ${YELLOW}CLAUDE_API_KEY${NC} set in the .env file if using Claude provider."
