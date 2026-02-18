# Verostark Engine 🛡️

![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)
![Rust](https://img.shields.io/badge/built_with-Rust-dca282.svg)
![Pingora](https://img.shields.io/badge/Powered_by-Pingora-orange)
![PostgreSQL](https://img.shields.io/badge/Database-PostgreSQL-blue)

**Verostark** is a high-performance, privacy-focused AI Gateway & Reverse Proxy written in Rust. It intercepts traffic between your applications and public AI providers (OpenAI, Anthropic, Google Gemini, etc.) to enforce strict PII scrubbing, audit logging, and security policies *before* data leaves your infrastructure.

> **"Your data, your rules. Even in the cloud."**

## 🚀 Key Features

*   **⚡ Ultra-Low Latency Proxy**: Built on Cloudflare's **Pingora** framework for async, multi-threaded performance.
*   **🔒 Dual-Layer PII Scrubbing**:
    *   **Reflex (Regex)**: Instant redaction of credit cards, social security numbers (SE), emails, and phone numbers.
    *   **Cortex (Local AI)**: Embedded BERT model (`rust-bert` / `candle`) detects context-sensitive entities like names and locations running locally on CPU.
*   **📝 Durable Audit Trail**: Full request/response logging to **PostgreSQL** with ACID compliance.
*   **🔀 Multi-Provider Routing**: Seamlessly route to OpenAI, Anthropic, Google, Mistral, or xAI using a single endpoint.
*   **🛡️ Outflow Guard**: Scans LLM responses to prevent PII leakage *from* the model back to the user.

## 🏗️ Architecture

Verostark sits as a sidecar or gateway in your VPC.

```mermaid
graph TD
    Client[Application/User] -->|HTTP Request| Verostark
    subgraph "Verostark Engine"
        Gatekeeper[Auth & Rate Limit]
        Scrubber[PII Redaction (Regex + AI)]
        Audit[Audit Logger]
    end
    Verostark -->|Scrubbed Request| Providers
    Providers[OpenAI / Anthropic / Gemini] -->|Response| Verostark
    Verostark -->|Logged Response| Client
    Audit -.->|Async Write| DB[(PostgreSQL)]
```

## 🛠️ Getting Started

### Prerequisites

*   **Rust** (latest stable)
*   **Docker** (for Postgres & release builds)
*   **PostgreSQL** (v15+)

### Installation

1.  **Clone the repository**:
    ```bash
    git clone https://github.com/verostark/engine.git
    cd engine
    ```

2.  **Setup Configuration**:
    Copy the example environment file:
    ```bash
    cp .env.example .env
    # Edit .env to set your DATABASE_URL
    ```

3.  **Start Database (Dev Mode)**:
    Use the helper script to spin up a local Postgres instance:
    ```bash
    ./dev_postgres.sh
    ```

4.  **Run the Engine**:
    ```bash
    cargo run --bin verostark_engine
    ```
    *The engine will automatically run database migrations on startup.*

## 💻 Usage

Change your application's `base_url` to point to Verostark (default: `http://localhost:8000`).

**Example: OpenAI Python Client**
```python
from openai import OpenAI

client = OpenAI(
    api_key="your-openai-key",
    base_url="http://localhost:8000/v1" 
)

# Use headers to route to specific providers
response = client.chat.completions.create(
    model="gpt-4",
    messages=[{"role": "user", "content": "Hello!"}],
    extra_headers={"X-AI-Provider": "openai"}
)
```

### Supported Providers (`X-AI-Provider`)
*   `openai` (Default)
*   `claude` (Anthropic)
*   `gemini` (Google)
*   `mistral` (Mistral AI)
*   `grok` (xAI)

## 🤝 Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for details on how to set up your development environment and submit PRs.

## 📄 License

This project is licensed under the **MIT License**. See the [LICENSE](LICENSE) file for details.
