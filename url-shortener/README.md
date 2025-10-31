1- docker compose up -d (starts Postgres, Redis, and the worker)

2- export $(cat .env | xargs) (or set envs manually)

3- cargo run (runs Axum locally)