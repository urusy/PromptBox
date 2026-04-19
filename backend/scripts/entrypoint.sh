#!/bin/sh
set -e

# Ensure DB is migrated before starting the API. Handles two cases:
#  1) Fresh DB initialized via db/init/*.sql (tables exist, no alembic_version)
#     → stamp baseline so upgrade head is a no-op
#  2) Already-migrated DB (alembic_version exists) → just upgrade head
python -m scripts.migrate

echo "[entrypoint] starting uvicorn"
exec uvicorn app.main:app --host 0.0.0.0 --port 8000
