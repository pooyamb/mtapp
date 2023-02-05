# An authentication and user management application

Part of one of my hobby projects ported from actix to axum. Although it is not tested at all and not even documented that much and is not production ready in any way, and misses some basic stuff like logging, it might be fun as a basis for your next great idea.

## Stack

- Axum web framework
- Sqlx for database connection
- Postgresql for db
- A hashmap in-memory KV store for jwt invalidation(switchable)
- Utoipa for openapi generation

## Tools I used

- Clap command line framework
- (a fork of) Schemer for migrations
- Sodiumoxide Argon2 for password hashing
- Many others I don't remember

## How to run

Do the basic rust installation and edit the .env.sample file.

To run the migrations use:

```
cargo migrate
```

To run the web app use the normal:

```
cargo run 
```

Migrations are stored in the final binary too, so you can run:

```
cargo run migrate
```

To create a user from terminal use:

```
cargo run mtapp-user create_user
```

And to assign roles(grant scopes) use:

```
cargo run mtapp-grant modify <username>
```

## OpenApi docs

There are 2 sets of docs available for public/internal apis.

Public facing apis are served on:
```
http://{host}:{port}/api/dev/docs/
```

Internal apis are served on:
```
http://{host}:{port}/api/internals/docs/
```

## Add new apps

Each app should have its own sub-crate, so:

```
cargo new --lib mynewapp
```

don't forget to add it into Cargo.toml, both in workspace section and in patches.

Then make an `app.rs` module and make an app struct(see mtapp-user for example). And impl `mtapp::App` methods depending on what it is gonna do. Both web routes and terminal commands can be defined there.

For migrations, create a `migrations` directory in the sub-crate, and make one folder for each migration. Folder's name is the migration's name, `up.sql` and `down.sql` files are for sql and `.meta.json` file should include dependencies and description.

At the end, mount your app in the src/main.rs and that's it.