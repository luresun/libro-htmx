#!/bin/sh
set -eu

if [ -n "${DATABASE_URL:-}" ]; then
  echo "Running SQL migrations..."
  sqlx migrate run
else
  echo "DATABASE_URL is not set; skipping SQL migrations."
fi

exec /app/server
