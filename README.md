# hookcatch

A local, self-hosted webhook receiver: binds a real HTTP server and logs
every incoming request — method, path, headers, body — to the terminal
in real time. A webhook.site/RequestBin alternative you run yourself,
for debugging what a third-party service (Stripe, GitHub, a payment
processor) is actually sending you, without a request ever leaving your
own machine or landing on someone else's server first.

## Installation

```sh
cargo install --path .
```

## Usage

```bash
hookcatch                # listens on 0.0.0.0:8787
hookcatch --port 9000
```

Point the webhook sender at `http://<your-host>:<port>/anything` (via a
tunnel like `ngrok`/`cloudflared` if the sender needs to reach you from
the public internet) and every request prints as it arrives. Every
request gets a plain `200 OK` back, since most webhook senders only care
that *something* acknowledged receipt.

## What gets shown

The request line (method, path with query string, HTTP version), every
header in the order they arrived, and the body — decoded as text if it's
valid UTF-8 (the overwhelming majority of real webhook payloads are
JSON), or shown as a byte count if it's binary.

## Status: built and verified against a real server, real sockets, and real curl requests

- **11 unit tests** (`cargo test --lib`): the HTTP/1.1 parser (request
  line, multiple headers in arrival order, a `Content-Length`-delimited
  body, case-insensitive `Content-Length` lookup, a malformed
  no-colon header line skipped rather than erroring, no body when
  `Content-Length` is absent) and the display formatter (method/path/
  version always shown, every header included, a UTF-8 body shown as
  text, a non-UTF-8 body shown as a byte count instead of garbage, no
  spurious blank body section when there's no body).
- **Live-verified against a real running server and real `curl`
  requests**, not just the parser in isolation: started the actual
  compiled binary listening on a real local port, sent a real `curl -X
  POST` with custom headers (`X-Signature`, `Content-Type`) and a real
  JSON body, and a real `curl GET` with a query string, from a separate
  process. Both were genuinely captured with the exact real headers
  `curl` sent (including its own `User-Agent`), the exact real query
  string on the path, and the JSON body decoded correctly — confirmed by
  reading the actual captured terminal output, not just checking the
  server started successfully.

**Not done / deliberately deferred**: HTTPS/TLS termination (this is a
plain-HTTP debugging tool, meant to sit behind a tunnel if public HTTPS
access is needed, the same assumption most local webhook-catcher tools
make); persisting captured requests to disk for later replay (this only
prints live — this workspace's `mockreplay` is the tool for record/replay
workflows); and chunked transfer-encoding request bodies aren't
supported, only `Content-Length`-delimited ones, which is what every
mainstream webhook sender actually uses.
