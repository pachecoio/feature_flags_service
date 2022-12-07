# Feature flags manager

A feature flags manager solution using Rust and MongoDB.


## Get started

### Requirements

To run the application, you need to have rust and cargo installed locally.
To install both, visit the [Rust website](https://www.rust-lang.org/tools/install) and follow the installation instructions.


#### Create a .env file with the MongoDB URI and Database name

```
touch .env
```
```
MONGODB_URI=<YOUR MONGODB URI>
DATABASE_NAME=<YOUR DB NAME>
```

#### Run the application with cargo:

```
cargo run
```

#### Run with Docker

Update the docker-compose.yaml file with your desired specifications and run with docker compose
```
docker compose up -d
```



#### Run the tests

```
cargo test
```

