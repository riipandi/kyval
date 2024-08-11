# Kyval

[![Crates version](https://img.shields.io/crates/v/kyval)](https://crates.io/crates/kyval)
[![Rust version](https://img.shields.io/badge/rust-v1.79-blue.svg?logo=rust&label=MSRV)](https://www.rust-lang.org)
[![Crates.io Total Downloads](https://img.shields.io/crates/d/kyval)](https://crates.io/crates/kyval)
[![Contribution welcome](https://img.shields.io/badge/Contributions-welcome-gray.svg)](https://github.com/riipandi/kyval/pulse)

---

Kyval is a simple key-value store based on LibSQL. This project is a fork of [Kyval Rust][keyv-rust],
originally created by [Christian Llontop][chrisllontop]. By utilizing LibSQL, Kyval offers a lightweight
and flexible alternative for simple data storage needs.

**Main changes:** The primary difference between Kyval and the original Kyval Rust is the use of
LibSQL as the database backend, replacing SQLite. This change enables Kyval to have improved
flexibility in data storage.

## Usage

### Instalation

```sh
cargo add kyval
```

### Interacting with Store

```rust
use kyval::Kyval;

#[tokio::main]
async fn main() {

    let kyval_store = KyvalStoreBuilder::new()
        .uri(":memory:")
        .table_name("kv_store")
        .build()
        .await.unwrap();

    let keyv = Kyval::try_new(kyval_store).await.unwrap();

    kyval.set("number", 42).await.unwrap();
    kyval.set("number", 10).await.unwrap();
    kyval.set("array", vec!["hola", "test"]).await.unwrap();
    kyval.set("string", "life long").await.unwrap();

    match kyval.get("number").await.unwrap() {
        Some(number) => {
            let number: i32 = serde_json::from_value(number).unwrap();
            assert_eq!(number, 10);
        }
        None => assert!(false),
    }

    match kyval.get("string").await.unwrap() {
        Some(string) => {
            let string: String = serde_json::from_value(string).unwrap();
            assert_eq!(string, "life long");
        }
        None => assert!(false),
    }

    match kyval.get("array").await.unwrap() {
        Some(array) => {
            let array: Vec<String> = serde_json::from_value(array).unwrap();
            assert_eq!(array, vec!["hola".to_string(), "test".to_string()])
        }
        None => assert!(false),
    }

    match kyval.remove_many(&["number", "string"]).await {
        Ok(_) => {}
        Err(_) => assert!(false),
    }
}
```

## License

Licensed under either of [Apache License 2.0][license-apache] or [MIT license][license-mit] at your option.

```plaintext
Unless you explicitly state otherwise, any contribution
intentionally submitted for inclusion in this project by you,
as defined in the Apache-2.0 license, shall be dual licensed
as above, without any additional terms or conditions.
```

Copyrights in this project are retained by their contributors.

See the [LICENSE-APACHE](./LICENSE-APACHE) and [LICENSE-MIT](./LICENSE-MIT) files
for more information.

[license-mit]: https://choosealicense.com/licenses/mit/
[license-apache]: https://choosealicense.com/licenses/apache-2.0/
[riipandi-twitter]: https://twitter.com/intent/follow?screen_name=riipandi
[riipandi-sponsors]: https://github.com/sponsors/riipandi
[keyv-rust]: https://github.com/chrisllontop/keyv-rust
[chrisllontop]: https://github.com/chrisllontop

---

<sub>🤫 Psst! If you like our work you can support us via [GitHub sponsors][riipandi-sponsors].</sub>

[![Made by](https://badgen.net/badge/icon/Made%20by%20Aris%20Ripandi?icon=bitcoin-lightning&label&color=black&labelColor=black)][riipandi-twitter]
