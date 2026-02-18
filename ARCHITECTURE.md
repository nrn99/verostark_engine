# Verostark Architecture

Verostark is designed as a high-performance, privacy-first AI Gateway. This document details its internal components and data flow.

## High-Level Overview

Verostark sits between your internal applications and external AI providers. It acts as a **transparent reverse proxy** that inspects and modifies traffic on the fly.

### Core Components

1.  **The Proxy (Pingora)**:
    *   Utilizes Cloudflare's `pingora` framework.
    *   Handles connection pooling, TLS termination, and asynchronous I/O.
    *   **File**: `src/main.rs` (implements `ProxyHttp` trait).

2.  **The Cortex (Smart Scrubbing)**:
    *   A local inference engine running a BERT-based Named Entity Recognition (NER) model.
    *   Uses `huggingface/candle` for efficient CPU inference.
    *   Detects `PERSON`, `LOCATION`, `ORGANIZATION`.
    *   **File**: `src/cortex.rs`.

3.  **The Reflex (Fast Scrubbing)**:
    *   A regex-based scrubber for deterministic PII (Credit Cards, Social Security Numbers, etc.).
    *   Runs *before* the Cortex to reduce load.
    *   **File**: `src/scrubber.rs`.

4.  **The Backpack (Context State)**:
    *   A request-scoped struct (`VerostarkContext`) that travels with the request lifecycle.
    *   Accumulates metadata: `request_id`, `client_ip`, `scrub_count`, `risk_verdict`.
    *   **File**: `src/main.rs`.

5.  **Persistence Layer**:
    *   Asynchronous audit logging to PostgreSQL using `sqlx`.
    *   Ensures durability of all processed requests.
    *   **File**: `src/db.rs`.

## Request Lifecycle

The life of a request through Verostark:

1.  **Ingress (Valve 0)**:
    *   Request arrives at port `8000`.
    *   **Rate Limiter**: Checks IP against in-memory rate limits (10 req/s).
    *   **Gatekeeper**: Validates `ALLOWED_IPS`.

2.  **Inspection (Valve 1)**:
    *   **Reflex Scrub**: Body is scanned with Regex.
    *   **Cortex Scrub**: Body is tokenized and run through BERT.
    *   **Sanitization**: PII is replaced with tokens (e.g., `<SE_PERSONNUMMER>`).
    *   **Modification**: `Content-Length` header is removed; `Transfer-Encoding: chunked` is set.

3.  **Routing**:
    *   `X-AI-Provider` header determines the upstream host (e.g., `api.anthropic.com`).
    *   `Host` header is rewritten to match upstream.

4.  **Egress (Upstream)**:
    *   Request is sent to the AI Provider.

5.  **Response (Valve 2)**:
    *   **Outflow Scrub**: The LLM's response is scanned for PII leakage.
    *   **Stamping**: A cryptographic-like JSON stamp is appended to the response body confirming verification.

6.  **Audit (Valve 3)**:
    *   Connection closes.
    *   **Log**: Structured JSON is printed to STDOUT.
    *   **Persist**: Audit record is INSERTED into PostgreSQL.

## Directory Structure

```
verostark_engine/
├── src/
│   ├── main.rs       # Entry point & Proxy Logic
│   ├── cortex.rs     # AI Model Logic (Candle)
│   ├── scrubber.rs   # Regex Logic
│   ├── db.rs         # Database Interaction
│   └── telemetry.rs  # In-memory monitoring
├── policies/         # Lua scripts for dynamic rules
├── migrations/       # SQL migration files
├── model/            # Local model weights (downloaded via script)
└── Cargo.toml        # Dependencies
```
