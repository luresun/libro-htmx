#!/bin/sh
set -e

echo "Running SQL migrations..."
sqlx migrate run

echo "Starting server..."
exec /app/server
