# CMS Rust

```bash
cargo install sea-orm-cli@^1.0

sea-orm-cli generate entity \
    --database-url postgres://cmsrust:cmsrust@localhost/cmsrustdb \
    --output-dir src/entities \
    --with-serde both \
    --expanded-format

cargo run
```
