#!/bin/bash

# Test script to validate environment variable loading
echo "Testing Environment Configuration"
echo "================================="

# Source the .env.sample file to test variables
if [ -f .env.sample ]; then
    echo "✓ .env.sample file exists"
    
    # Count the number of configuration variables
    VAR_COUNT=$(grep -c "^[A-Z]" .env.sample)
    echo "✓ Found $VAR_COUNT environment variables"
    
    # Check for required variables
    REQUIRED_VARS=("MASTER_KEY" "GLOBAL_PB_URL" "GLOBAL_PB_ADMIN_EMAIL" "GLOBAL_PB_ADMIN_PW" "RUST_LOG" "QUEUE_CONCURRENCY")
    
    for var in "${REQUIRED_VARS[@]}"; do
        if grep -q "^$var=" .env.sample; then
            echo "✓ $var is present"
        else
            echo "✗ $var is missing"
        fi
    done
    
    # Check for strong password
    if grep -q "GLOBAL_PB_ADMIN_PW=.*[!@#$%^&*()_+]" .env.sample; then
        echo "✓ Admin password contains special characters"
    else
        echo "! Admin password should contain special characters"
    fi
    
    # Check for base64 keys
    if grep -q "MASTER_KEY=.*[A-Za-z0-9+/=]" .env.sample; then
        echo "✓ Master key appears to be base64 encoded"
    else
        echo "! Master key should be base64 encoded"
    fi
    
else
    echo "✗ .env.sample file not found"
    exit 1
fi

echo ""
echo "Testing Backend Configuration"
echo "============================="

# Test backend compilation
if cargo check -p backend --quiet; then
    echo "✓ Backend compiles successfully"
else
    echo "✗ Backend compilation failed"
fi

echo ""
echo "Testing Worker Configuration"
echo "============================"

# Test worker compilation
if cargo check -p worker --quiet; then
    echo "✓ Worker compiles successfully"
else
    echo "✗ Worker compilation failed"
fi

echo ""
echo "Testing Frontend Configuration"
echo "=============================="

# Test frontend compilation
if cargo check -p fathom-loom-frontend --quiet; then
    echo "✓ Frontend compiles successfully"
else
    echo "✗ Frontend compilation failed"
fi

echo ""
echo "Configuration Test Complete"
echo "=========================="
