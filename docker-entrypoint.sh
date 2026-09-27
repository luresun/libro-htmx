#!/bin/sh
set -e

echo "Starting application..."
echo "DATABASE_URL is set: ${DATABASE_URL:+yes}"
echo "JWT_SECRET is set: ${JWT_SECRET:+yes}"

# Start the server
exec /app/server
