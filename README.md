# Brightspace CLI

A command-line interface for interacting with the Brightspace API.

## Design

This project is a Rust-based CLI application that provides a simple and efficient way to interact with the Brightspace API. It is designed to be a lightweight and fast alternative to the web interface for common tasks.

The application uses a layered architecture:
- The command-line interface is built using the `clap` crate.
- The application logic is separated into modules for handling the API, configuration, and commands.
- The Brightspace API is accessed through a dedicated `BrightspaceApi` struct that handles requests and authentication.
- Configuration is loaded from a YAML file located at `~/.config/brightspace/brightspace-cli.yaml`. A local `brightspace-cli.yaml` can be used for development.

## Releases

Pre-built binaries for supported platforms can be found in the [releases](releases/) directory or on the GitHub Releases page (coming soon).

See [RELEASES.md](RELEASES.md) for detailed installation and build instructions.

## Installation

To install the CLI, you can build it from source using Cargo:

```bash
cargo install --path .
```

This will install the `brightspace-cli` binary to your Cargo bin directory (usually `~/.cargo/bin`). Ensure this directory is in your system's `PATH`.

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

## Usage

This section provides a detailed reference for all the available commands.

### Configuration

The `config` subcommand is used to manage thebrightspace-cli configuration file.

#### Initialize Configuration

To create a defaultbrightspace-cli configuration file, run the `config init` command:

```bash
brightspace-cli config init
```

This will create a `brightspace-cli.yaml` file at `~/.config/brightspace/brightspace-cli.yaml`.

#### Get Configuration Values

To get a value from thebrightspace-cli configuration file, use the `config get` command:

```bash
brightspace-cli config get client_id
```

#### Set Configuration Values

To set a value in thebrightspace-cli configuration file, use the `config set` command:

```bash
brightspace-cli config set client_id <your_client_id>
brightspace-cli config set username <your_username>
```

### Help

To get help with any command or subcommand, you can use the `help` command or the `-h` and `--help` flags.

```bash
brightspace-cli help
brightspace-cli --help
brightspace-cli configbrightspace-cli --help
brightspace-cli helpbrightspace-cli config
```

### Authentication

The `auth` subcommand is used to managebrightspace-cli authentication.

#### Login

Tobrightspace-cli authorize the application and get an access token, run the `auth login` command:

```bash
brightspace-cli auth login
```

#### Logout

To delete the stored access and refresh tokens, run the `auth logout` command:

```bash
brightspace-cli auth logout
```

### Commands

####brightspace-cli whoami

The `whoami` command fetches and displays information about the current user using the stored access token.

```bash
brightspace-cli whoami
```

#### convert

The `convert` command converts a Quarto Markdown (`.qmd`) file to a QTI zip file. This command uses a native Rust implementation and does not require any external dependencies.

To convert a file, use the `convert` command with the path to the file:

```bash
brightspace-cli convert /path/to/your/quiz.qmd
```

##### Quiz File Format

The tool uses a Quarto-style Markdown format with YAML front matter and fenced divs for questions.

**Example:**

```markdown
---
title: Java II Programming Concepts
description: A quiz covering intermediate Java topics.
shuffle_answers: true
---

:::{.question type=multiple_choice points=1}
Which statement is true about abstract classes in Java?

- [ ] An abstract class can be instantiated directly.
- [x] An abstract class can have both abstract and non-abstract methods.
- [ ] All methods in an abstract class must be abstract.
:::
```

**Question Attributes:**

You can set the following attributes in the question fence `:::{.question ...}`:

- `type`: The type of question (required).
- `points`: The point value for the question (default: 1).
- `title`: A short title for the question (optional). If not provided, a truncated version of the prompt is used.

**Supported Question Types:**

Here are examples for each supported question type.

**Multiple Choice (`multiple_choice`)**
Select one correct answer.

```markdown
:::{.question type=multiple_choice points=1 title="Abstract Classes"}
Which statement is true about abstract classes in Java?

- [ ] An abstract class can be instantiated directly.
- [x] An abstract class can have both abstract and non-abstract methods.
- [ ] All methods in an abstract class must be abstract.
:::
```

**Multiple Answers (`multiple_answers`)**
Select all correct answers.

```markdown
:::{.question type=multiple_answers points=2}
Which of the following collections in Java do not allow duplicate elements?

- [x] Set
- [ ] List
- [x] HashSet
- [ ] ArrayList
:::
```

**True/False (`true_false`)**
A simple true or false question.

```markdown
:::{.question type=true_false points=1}
In Java, a class can inherit from multiple parent classes.

- [ ] True
- [x] False
:::
```

**Short Answer (`short_answer`)**
The user must type the exact answer.

```markdown
:::{.question type=short_answer points=1}
What keyword is used to manually throw an exception in Java?

- [x] throw
:::
```

**Essay (`essay`)**
An open-ended text response.

```markdown
:::{.question type=essay points=5}
Explain the concept of polymorphism in object-oriented programming.
:::
```

**File Upload (`file_upload`)**
The user must upload a file.

```markdown
:::{.question type=file_upload points=5}
Upload your source code file here.
:::
```

### Math Support

The tool supports LaTeX math expressions, which are automaticallybrightspace-cli converted to MathML for QTI compatibility.

-   **Inline Math**: Wrap your LaTeX in single dollar signs `$ ... $`.
-   **Block Math**: Wrap your LaTeX in double dollar signs `$$ ... $$`.

**Example:**

```markdown
:::{.question type=multiple_choice points=1}
Solve for $x$: $x^2 - 4 = 0$

- [x] $x = \pm 2$
- [ ] $x = 4$
- [ ] $x = 2$
:::
```

### Music Support

The tool supports MusicXML notation using the `verovio` command-line tool.

> [!IMPORTANT]
> **Requirement**: You must have [Verovio](https://verovio.org) installed and available in your system PATH to generate music notation images.
> The source code for Verovio is available at [https://github.com/rism-digital/verovio](https://github.com/rism-digital/verovio).

If `verovio` is installed, `musicxml` code blocks will be automaticallybrightspace-cli converted to SVG images. If not, they will be rendered as code blocks.

**Example:**

````markdown
:::{.question type=multiple_choice points=1}
Identify this scale:

```musicxml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE score-partwise PUBLIC "-//Recordare//DTD MusicXML 3.1 Partwise//EN" "http://www.musicxml.org/dtds/partwise.dtd">
<score-partwise version="3.1">
  ...
</score-partwise>
```

- [x] C Major
- [ ] G Major
:::
````

