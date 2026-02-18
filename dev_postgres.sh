#!/bin/bash
set -e

CONTAINER_NAME="verostark-dev-db"
DB_USER="verostark"
DB_PASS="password"
DB_NAME="verostark"
DB_PORT="5432"

echo "🐘 Starting Local PostgreSQL for Verostark Testing..."

# Cleanup previous run
if docker ps -a | grep -q $CONTAINER_NAME; then
    echo "Stopping removing existing container..."
    docker rm -f $CONTAINER_NAME
fi

# Run Postgres
docker run --name $CONTAINER_NAME \
    -e POSTGRES_USER=$DB_USER \
    -e POSTGRES_PASSWORD=$DB_PASS \
    -e POSTGRES_DB=$DB_NAME \
    -p $DB_PORT:5432 \
    -d postgres:15-alpine

echo "⏳ Waiting for Database to be ready..."
sleep 3 # Give it a moment to initialize

echo "✅ Database Running!"
echo "   URL: postgres://$DB_USER:$DB_PASS@localhost:$DB_PORT/$DB_NAME"
echo ""
echo "To run Verostark with this DB:"
echo "   export DATABASE_URL=postgres://$DB_USER:$DB_PASS@localhost:$DB_PORT/$DB_NAME"
echo "   cargo run --bin verostark_engine"
echo ""
echo "To query manually:"
echo "   docker exec -it $CONTAINER_NAME psql -U $DB_USER -d $DB_NAME"
