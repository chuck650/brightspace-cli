# Brightspace CLI Concept

## Purpose

A command-line interface for interacting with the Brightspace API.

## High-Level Design

The application is a Rust-based CLI that uses a subcommand structure to organize its functionality.

### Configuration

Configuration is stored in a YAML file located at `~/.config/brightspace/brightspace-cli.yaml`. A local `brightspace-cli.yaml` can be used for development.

The `config` subcommand is used to manage the configuration file. It has the following subcommands:
- `init`: Creates a new configuration file with default values.
- `get <KEY>`: Gets a value from the configuration file.
- `set <KEY> <VALUE>`: Sets a value in the configuration file.

The configuration file stores the following information:
- `base_url`: The base URL of the Brightspace instance.
- `client_id`: The application's client ID for the Brightspace API.
- `client_secret`: The application's client secret for the Brightspace API.
- `username`: The username to use for authentication.
- `redirect_uri`: The redirect URI for the OAuth 2.0 flow.
- `auth_url`: The authorization URL for the OAuth 2.0 flow.
- `token_url`: The token URL for the OAuth 2.0 flow.

### Authentication

The `auth` subcommand is used to manage authentication. It has the following subcommands:
- `login`: Initiates the OAuth 2.0 flow to get an access token.
- `logout`: Deletes the stored access and refresh tokens.

Access and refresh tokens are stored securely in the system's keychain.

### API Interaction

The CLI interacts with the Brightspace API through a dedicated `BrightspaceApi` struct. This struct is responsible for:
- Making authenticated requests to the API using an OAuth 2.0 access token.
- Handling responses and errors.
- Deserializing API responses into Rust structs.
