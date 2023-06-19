# Migrations and Entity actions

### Database Migration configuration

1. Configuring sea-orm-cli

```shell
cargo install sea-orm-cli
```

2. Be sure to be located inside `src` route'

```sh
cd src
```

3. Creating migration files

```shell
sea-orm-cli migrate generate public_directory #[--local-time]
```

4. Add your content for that migration file, for example in the previous example a template for public-directory entity was created, so now you can customize its content accoding to the entity model it should be.

## Additional commands

# Running Migrator CLI

- Generate a new migration file
  ```sh
  cargo run -- migrate generate MIGRATION_NAME
  ```
- Apply all pending migrations
  ```sh
  cargo run
  ```
  ```sh
  cargo run -- up
  ```
- Apply first 10 pending migrations
  ```sh
  cargo run -- up -n 10
  ```
- Rollback last applied migrations
  ```sh
  cargo run -- down
  ```
- Rollback last 10 applied migrations
  ```sh
  cargo run -- down -n 10
  ```
- Drop all tables from the database, then reapply all migrations
  ```sh
  cargo run -- fresh
  ```
- Rollback all applied migrations, then reapply all migrations
  ```sh
  cargo run -- refresh
  ```
- Rollback all applied migrations
  ```sh
  cargo run -- reset
  ```
- Check the status of all migrations
  ```sh
  cargo run -- status
  ```
