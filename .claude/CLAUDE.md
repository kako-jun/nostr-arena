# nostr-arena

Nostr-based multiplayer game arena library.

## 技術スタック

- Rust 2024 (core)
- wasm-bindgen (npm bindings)
- PyO3/maturin (Python bindings)
- nostr-sdk 0.38

## 構造

```
crates/nostr-arena-core/  # Rust core
bindings/wasm/            # npm (WASM)
bindings/python/          # PyPI
examples/                 # 使用例
docs/                     # 詳細ドキュメント
```

## ビルド

```bash
cargo build              # Rust
cd bindings/wasm && wasm-pack build  # npm
cd bindings/python && maturin build  # PyPI
```

## ドキュメント

| ファイル | 内容 |
|---------|------|
| [docs/protocol.md](../docs/protocol.md) | Nostrイベント仕様 |
| [docs/architecture.md](../docs/architecture.md) | アーキテクチャ |
