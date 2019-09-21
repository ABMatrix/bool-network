# Caster

For making move transaction and assist.

## Accounts
privete: 0x4db4ef1992889d4428e400be3428843db6e89bb2e8aaf4ce8efe00df64012544
public: 0x01add5624932fc6e5e82ea4b8b4217c2ea4372a1e4fbc9d910a38b2514931166
address: 0x44416e28b8545d375a212c44d9719e5c21c4f44123be4993768c899bf3c02826

private: 0x4db4ef1992889d4428e400be3428843db6e89bb2e8aaf4ce8efe00df6401253c
public: 0xd0b56296877f8acefdefef06569751d8587f8f5df255179957086012b4fb7d20
address: 0xb2c5ac79fdc6f4b8159a0500104ec59c99c5413a52423bfb2d23bc43290c6907

# Demo
## Mint
```bash
cargo run -- tx -m mint -k 0x4db4ef1992889d4428e400be3428843db6e89bb2e8aaf4ce8efe00df64012544 -r 0x44416e28b8545d375a212c44d9719e5c21c4f44123be4993768c899bf3c02826 -v 10000 -s 0
```
## Transfer
```bash
cargo run -- tx -m transfer -k 0x4db4ef1992889d4428e400be3428843db6e89bb2e8aaf4ce8efe00df64012544 -r 0xb2c5ac79fdc6f4b8159a0500104ec59c99c5413a52423bfb2d23bc43290c6907 -v 100 -s 0
```
## Publish Module
```bash
cargo run -- tx -k 0x4db4ef1992889d4428e400be3428843db6e89bb2e8aaf4ce8efe00df64012544 -m publish --compiled_file ./scripts/m.mvir -s 1

```
## Call Module
```bash
cargo run -- tx -k 0x4db4ef1992889d4428e400be3428843db6e89bb2e8aaf4ce8efe00df64012544 -m publish --compiled_file ./scripts/s.mvir -s 2
```