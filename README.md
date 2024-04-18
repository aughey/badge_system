# badge_system

# Capabilities

Two visible components, 

- a physical badge in the form of a Badger 2040W.
- a publicly accessible web page

## General functionality

- On web interface
    - Web page displays a preview rendering of what the badge should look like.
    - Text boxes to update the text of the badge.
    - Drop down control to change the period of the flashing LED.
    - Button to send this state to the badge.
- On the badge
    - Badge will initialize hardware and wifi
    - Obtain an IP address over wifi/dhcp to a pre-defined access point
    - Connect to the public server
    - Communicate with that server to receive badge updates
    - Badge update consist of
         - New text to display
         - Flash rate of the LED

# Narriative

The purpose of this is to showcase/demonstrate Rust running on multiple devices with a 100% rust codebase.  This will show rust running in a bare-metal no_std embedded configuration, a fully Linux hosted web server, and wasm on a web browser.  Additionally, a wide range of technologies will be utilitized such as encryption/authenticaion with TLS.

## Features

(aside)

- Development experience
    - Project Management (design practices, gate reviews, design-code-test cycles, documentation, requirements definition)
    - People Management (version one, standups, code reviews, design reviews)
    - Development environment (IDE, dev-containers, testing env, build environment)
    - Development experience (workflow, build-test cycle, unit interaction, emulation)
    - Coding conceptual abstractions (architecture, async/sync, language abstractions, language enforcement/constraints)
    - Artifact distribution (customer interaction, deployment mechanisms)
    - Execution Environments (operating environments)

- Bare-metal environment
     - RP2040W microcontroller
     - Cortex-M0 processor (32-bit RISC ARM)
     - thumbv6m-none-eabi target [reduced instruction set encoding](https://stackoverflow.com/questions/28669905/what-is-the-difference-between-the-arm-thumb-and-thumb-2-instruction-encodings)
     - [Embassy asynchronous framework](https://embassy.dev/)
     - Async development/operating environment
     - Multi-core
     - no_std embedded tls library for authentication/encryption
- Web Client
    - [Leptos framework](https://leptos.dev/)
    - Rust code compiled to wasm
    - Preview display rendered with same code base
- Web Server
    - Leptos framework (same env as client)
    - Multi-threaded + async
    - Separate TLS stack for direct-badge comm (split https and badge-specific comm)
- Architecture
    - TLS 1.3 communication between badge and server
    - Badge authentication/revocation through Pki certificates
    - Encryption using TLS13_AES_256_GCM_SHA384, TLS13_AES_128_GCM_SHA256, or TLS13_CHACHA20_POLY1305_SHA256,
    - 


```
vscode ➜ /workspaces/badge_system (main) $ cargo run --release --package run-wasm -- --package basic_example
```

```
vscode ➜ /workspaces/badge_system/nginx (main) $ sudo nginx -p `pwd` -c ./nginx.conf 
```

```
vscode ➜ /workspaces/badge_system/web-badge (main) $ cargo leptos watch -- certs/CA_cert.crt  certs/server.crt certs/server.key
```

```
PS C:\jha\socat-1.7.3.2-1-x86_64> ./socat.exe TCP4-LISTEN:4444,fork,reuseaddr TCP4:127.0.0.1:4443
```