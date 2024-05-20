# DockStream Container Tool

DockStream is a command-line utility designed to facilitate the management and isolation of containerized applications, mimicking functionalities similar to Docker but with a streamlined focus on security and simplicity.

## Features

- **Image Pulling**: Efficiently pull images from Docker Hub, utilizing a secure authentication process.
- **Sandbox Environment Setup**: Establish a secure, isolated sandbox environment for running containerized applications.
- **Process Isolation**: Utilize Linux namespaces to ensure that each containerized application runs in its isolated process environment.
- **Command Execution**: Execute commands within the isolated environment, allowing for controlled and secure operations.

## Requirements

- Rust programming language
- Cargo package manager
- Access to Linux OS capabilities
- Reqwest and serde_json for HTTP requests and JSON handling

## Installation

1. **Clone the repository:**
   ```bash
   git clone https://github.com/botirk38/DockStream
   cd src
   ```

2. **Build the project:**
   ```bash
   cargo build --release
   ```

3. **Run the built executable:**
   ```bash
   ./src/your_docker.sh <args>
   ```

## Usage

To use DockStream, you need to provide a series of arguments that specify the image, the command to execute, and any command arguments. Here is the basic syntax:

```bash
mydocker run alpine:latest /usr/local/bin/docker-explorer mypid

```


## Contributing

Contributions are welcome! Please fork the repository and submit pull requests with your features and bug fixes.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.


