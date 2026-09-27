#!/bin/sh
set -eu

echo "Starting app..."

# Only run migrations if DATABASE_URL is set
if [ -n "${DATABASE_URL:-}" ]; then
  echo "Running SQL migrations..."
  
  # Check if sqlx CLI exists
  if command -v sqlx &> /dev/null; then
    sqlx migrate run
  else
    echo "Warning: sqlx CLI not found, skipping migrations."
    echo "Migrations should be applied manually or via a different method."
  fi
else
  echo "DATABASE_URL is not set; skipping migrations."
fi

echo "Starting server..."
exec /app/server
