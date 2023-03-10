# Indexer Setup

## Setup

- install postgres, diesel
- setup db

## Adding a migration

- diesel migration generate migration-name-here
- Implement up.sql and down.sql
- `diesel migration run`
- add implementation to save event
- add case to worker.rs

## local dev

Use docker-compose for local test db

```
docker-compose up -d
# export DATABASE_URL=postgresql://postgres:test@localhost:5432/postgres
```
