# 📜 Soroban "Meet and pay" Smart Contract

This is a simple smart contract built using [Soroban](https://soroban.stellar.org), the smart contract platform for the Stellar blockchain. The contract demonstrates basic functionality by returning a greeting message using input provided to it.

---

## 📦 Features

- Minimal `no_std` contract written in Rust.
- Accepts a `String` input and returns a `Vec<String>` with a greeting.
- Serves as a template for more advanced smart contract development on Soroban.

---

## 🛠️ Contract Details

```rust
pub fn hello(env: Env, to: String) -> Vec<String>
```

### Parameters

- `env`: The Soroban environment (automatically injected).
- `to`: A `String` that represents the name of the recipient.

### Returns

- A vector containing the string `"Hello"` followed by the name provided.

### Example Output

```rust
vec!["Hello", "Alice"]
```

---

## 🧪 Testing

This contract comes with a `test.rs` module that demonstrates how to write unit tests for Soroban contracts.

To run tests:

```sh
cargo test
```

---

## 📚 Resources

- Soroban Developer Docs: [https://developers.stellar.org/docs/build/smart-contracts/overview](https://developers.stellar.org/docs/build/smart-contracts/overview)
- Soroban Examples Repo: [https://github.com/stellar/soroban-examples](https://github.com/stellar/soroban-examples)

---

## 🚀 Getting Started

If you're new to Soroban:

1. Install the Soroban CLI:
   [https://soroban.stellar.org/docs/install-soroban-cli](https://soroban.stellar.org/docs/install-soroban-cli)

2. Initialize a new project or clone this one.

3. Build and deploy your contract using the Soroban CLI.

---

## 📄 License

This project is open-sourced under the MIT License.

---

Let me know if you'd like to include CLI usage, contract deployment steps, or WASM generation instructions too.

