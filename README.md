<p align="center">
        <img src="assets/hikari_icon.jpg" width="200" height="200" style="border-radius: 50%" />
</p>
<h1 color="#000000" size="100px" align="center">hikari for Discord</h1>

hikari is a super fast, bare-minimum protection bot for Discord written in Rust. 

I built this to avoid bloated bots. It uses Postgres for storage and Redis to keep things snappy. You can use it to lock down your server, or just fork it as a clean starter template if you want to build your own Rust bot without the headache.

## Configuration

The application loads configuration via environment variables (usually from a `.env` file).

- `TOKEN` (required): Your Discord bot token.
- `DATABASE_URL` (required): Your PostgreSQL database connection URL.
- `REDIS_URL` (required): Your Redis server connection URL.
- `PREFIX` (required): Your bot's default text command prefix.

## Database setup (sqlx)

We use sqlx with PostgreSQL. Point `DATABASE_URL` at the database server you want to use (default example: `postgresql://user:pass@localhost:5432/hikari`). Run the following after changing schemas or when setting up a fresh checkout:

1. Create the database: `cargo sqlx database create`
2. Apply migrations: `cargo sqlx migrate run`

## Running locally (without containers)

1. Ensure `.env` is populated and `DATABASE_URL` / `REDIS_URL` point at your local PostgreSQL and Redis instances.
2. Run the sqlx commands above to get the schema ready.
3. Start the bot: `cargo run` (or use the VS Code task "cargo run").

## Running with Docker

You can run the bot directly using the pre-built image from the GitHub Container Registry. 

Ensure you have your `.env` file ready with the required variables, then simply run:
```sh
docker run -d --name hikari --env-file .env ghcr.io/lowkwrs/hikari:latest
```
*(Make sure your `DATABASE_URL` and `REDIS_URL` point to accessible instances from within the container.)*

## Credits

Built specifically for high performance, utilizing [twilight-rs](https://github.com/twilight-rs/twilight) and modern Rust architecture.