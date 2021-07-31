# nihondrill

> practice japanese

* Install rust
* `cargo install migrant --features postgres`
* `cargo install sqlx-cli`
* Create a `.env` by copying the `.env.sample`. The migration tool (`migrant`),
  the server application, and the `sqlx`  database library will all automatically
  apply any values listed in your `.env` to the current environment, so you don't
  need to "source" the .env manually.
* Setup a postgres db with the `DB_*` values listed in your env.
* `migrant setup`
* `migrant apply -a`
* `cargo run`, note that `sqlx` needs to see a `DATABASE_URL` (set in your `.env`)
  environment variable at compile time to validate database queries.
* `cargo sqlx prepare` to update the sqlx-data.json file

```
# as postgres user
# psql
create database nd;
create user nd with encrypted password 'nd';
grant all on database nd to nd;

# psql -d nd
create extension pg_trgm;
```
