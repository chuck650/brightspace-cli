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

## Workflow

This section describes the typical workflow for using the Brightspace CLI.

### 1. Register an Application in Brightspace

Before you can use this CLI, you need to register it as an application in your Brightspace instance. This will provide you with a `client_id` and a `client_secret`. These are standard credentials for the OAuth 2.0 authentication flow and are required to identify and authenticate your application.

**Note:** Registering an application in Brightspace requires administrator privileges. If you are not an administrator, you will need to contact your institution's Brightspace administrator to get a `client_id` and `client_secret`. For more information, see the official Brightspace documentation on [how to register an application](https://community.d2l.com/brightspace/kb/articles/4794-manage-extensibility-register-an-app).

Alternatively, your administrator can use the [keytool utility](https://docs.valence.desire2learn.com/admin/keytool.html) to generate a `client_id` and `client_secret` for you. This also requires special permissions.

1.  Log in to your Brightspace instance with an administrator account.
2.  Go to **Admin Tools** > **Manage Extensibility**.
3.  Click **Register an App**.
4.  Fill in the application details:
    *   **Application Name:** A descriptive name for your application (e.g., "Brightspace CLI").
    *   **Trusted:** Check this box.
    *   **Redirect URI:** The redirect URI for the OAuth 2.0 flow. For this CLI, you can use the default value of `http://localhost:8080`.
    *   **Major Version:** 1
    *   **Minor Version:** 0
    *   **Scope:** `core:*:*`
5.  Click **Register**.
6.  You will now see your `client_id` and `client_secret`. Keep these values safe, as you will need them to configure the CLI.

### 2. Configure the CLI

Use the `config set` command to set your `client_id`, `client_secret`, and `username`. You only need to do this once.

```bash
brightspace-cli config set client_id <your_client_id>
brightspace-cli config set client_secret <your_client_secret>
brightspace-cli config set username <your_username>
```

### 3. Log in

Use the `auth login` command to authorize the application. This will open a URL in your browser, and you will need to copy and paste an authorization code. You will need to do this once to get the initial access and refresh tokens. The application will handle refreshing the access token automatically.

```bash
brightspace-cli auth login
```

### 4. Use the CLI

You can now use the other commands, like `whoami`, to interact with the Brightspace API.

```bash
brightspace-cli whoami
```

