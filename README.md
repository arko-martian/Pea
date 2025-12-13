# 🫛 Pea — Blazingly Fast JS/TS Package Manager (Rust)

<img src="./peaman!.png" alt="Alt Text" width="500"/>






<br>
<br>
> **Pea** is a next‑generation JavaScript & TypeScript package manager written entirely in **Rust** 🦀.
> It focuses on **speed**, **stability**, **modularity**, and **real‑world resilience** — even under bad internet conditions.

Pea is designed to be **tiny**, **predictable**, and **future‑proof**, without betraying older systems or developers.

---

## ✨ Why Pea?

Modern JS package managers are fast — but fragile.

Pea is built with a different philosophy:

* ⚡ **Blazingly fast** (Rust core, minimal overhead)
* 🧩 **Modular architecture** (no domino‑effect failures)
* 🔒 **Deterministic installs** (lockfile‑first design)
* 🌐 **Network‑resilient installs** (handles Wi‑Fi drops safely)
* 📦 **Cache‑first & atomic installs** (never corrupt `node_modules`)
* 🦀 **Rust‑native tooling** (zero JS runtime dependency)

Pea aims to be a **clean, honest alternative** to npm, yarn, pnpm, and bun.

---

## 🚀 Features

### ✅ Implemented / In Progress

* CLI similar to npm/bun for easy adoption
* Project initialization (`new`, `init`)
* Dependency install flow (`install`, `add`, `remove`)
* Lockfile generation (`pea.lock`)
* Node‑style `node_modules` layout
* Structured logging & progress output
* Modular internal architecture

### 🧠 Planned / Upcoming

* Full dependency resolution
* Version & range solver
* Registry metadata caching
* Offline & resume‑safe installs
* Atomic extraction (crash & power‑safe)
* TOML lockfile support (high‑performance parser)
* Custom registry support
* Workspace / monorepo support

---

## 🧠 Network‑Resilient Installs (Unique Feature)

Unlike traditional package managers, Pea **detects internet loss during installation**.

If the connection drops mid‑install:

* Installation is **paused**, not corrupted
* User is prompted to **wait or terminate**
* Downloads safely **resume when internet returns**
* No half‑installed dependencies

This makes Pea reliable in real‑world conditions like:

* Load shedding
* Mobile hotspots
* Unstable Wi‑Fi

---

## 📦 Installation

> ⚠️ Pea is currently in **early development (v0.1.0)**.

```bash
# build from source
cargo build --release

# run locally
./target/release/pea --help
```

---

## 🧪 Usage

```bash
pea new my-project
cd my-project

pea install
pea add react
pea remove lodash

pea run dev
pea build
```

---

## 🧾 CLI Commands

```text
new       Create a new project
init      Initialize in current directory
install   Install dependencies
add       Add a dependency
remove    Remove a dependency
run       Run a script
build     Build for production
test      Run tests
check     Check configuration
publish   Publish a package
upgrade   Upgrade Pea
clean     Clean cache
version   Show version
```

---

## 🔒 Lockfile

Pea uses a deterministic lockfile (`pea.lock`) to guarantee reproducible installs.

Design goals:

* Minimal format
* Human‑readable
* Fast to parse
* Crash‑safe writes

> TOML support is planned for a future release.

---

## 🧩 Architecture Philosophy

Pea follows a strict internal rule:

> **One feature per module. No cascading failures.**

If one component fails:

* Others continue safely
* State remains consistent
* Errors are explicit and recoverable

This makes Pea easy to evolve and extremely robust.

---

## 🦀 Built With Rust

Why Rust?

* Memory safety
* Fearless concurrency
* Predictable performance
* Tiny binaries

Pea avoids unnecessary abstractions and focuses on **raw efficiency**.

---

## 🧭 Project Status

* Version: **v0.1.0**
* Status: **Active development**
* Stability: **Experimental**

Expect rapid iteration.

---

## 🤝 Contributing

Contributions, ideas, and discussions are welcome.

Guidelines:

* Keep modules small
* Prefer clarity over cleverness
* No hidden side effects
* Respect deterministic behavior

---

## 📜 License

MIT License

---

## 🌱 Vision

Pea is not just a package manager.

It is part of a larger Rust‑first ecosystem focused on:

* developer freedom
* performance without bloat
* long‑term maintainability

> **Tiny by design. Powerful by nature.** 🫛
