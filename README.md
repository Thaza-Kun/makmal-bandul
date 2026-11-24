# Makmal Bandul

Offline-first pendulum experiment lab (Rust → WASM), built with Yew and Rapier2D.

## Develop

```sh
trunk serve
```

## Test

```sh
cargo test --target x86_64-unknown-linux-gnu
```

## Build

```sh
trunk build --release
# GitHub Pages subpath:
trunk build --release --public-url /<repo-name>/
```

Output is in `dist/`. Pushing to `main`/`master` deploys it via GitHub Pages using `.github/workflows/deploy.yml`.

See `DESIGN.md` for the specification and `REVIEWS.md` for decisions and known limits.
