# 第三方组件与许可

Android 与 Windows 的安装包里带着下面这些第三方组件。它们各自的许可要求保留著作权与许可声明，这一页就是那些声明；许可正文在 [`LICENSES/`](LICENSES) 目录。

这份清单由 `cargo metadata --offline --format-version 1` 的解析结果生成，覆盖两个二进制里实际链接的全部 Rust 组件；Android 侧的 Kotlin 组件另列一节。

## 许可正文

| 许可 | 正文 |
|---|---|
| Apache-2.0 | [`LICENSES/Apache-2.0.txt`](LICENSES/Apache-2.0.txt) |
| MIT | [`LICENSES/MIT.txt`](LICENSES/MIT.txt) |

零散的 BSD-3-Clause / BSD-2-Clause / ISC / Unicode-3.0 / MPL-2.0 / Unlicense / Zlib / CDLA-Permissive-2.0 组件只要求保留声明，其许可正文随各自源码分发，下表给了每个组件的上游地址。

## Rust 组件（279 个）

| 组件 | 版本 | 许可 | 上游 |
|---|---|---|---|
| `adler2` | 2.0.1 | 0BSD OR MIT OR Apache-2.0 | [github.com/oyvindln/adler2](https://github.com/oyvindln/adler2) |
| `ahash` | 0.8.12 | MIT OR Apache-2.0 | [github.com/tkaitchuck/ahash](https://github.com/tkaitchuck/ahash) |
| `aho-corasick` | 1.1.5 | Unlicense OR MIT | [github.com/BurntSushi/aho-corasick](https://github.com/BurntSushi/aho-corasick) |
| `alloc-no-stdlib` | 3.0.0 | BSD-3-Clause | [github.com/dropbox/rust-alloc-no-stdlib](https://github.com/dropbox/rust-alloc-no-stdlib) |
| `alloc-stdlib` | 0.3.0 | BSD-3-Clause | [github.com/dropbox/rust-alloc-no-stdlib](https://github.com/dropbox/rust-alloc-no-stdlib) |
| `anyhow` | 1.0.104 | MIT OR Apache-2.0 | [github.com/dtolnay/anyhow](https://github.com/dtolnay/anyhow) |
| `arbitrary` | 1.5.0 | MIT OR Apache-2.0 | [github.com/rust-fuzz/arbitrary](https://github.com/rust-fuzz/arbitrary/) |
| `askama` | 0.13.1 | MIT OR Apache-2.0 | [github.com/askama-rs/askama](https://github.com/askama-rs/askama) |
| `askama_derive` | 0.13.1 | MIT OR Apache-2.0 | [github.com/askama-rs/askama](https://github.com/askama-rs/askama) |
| `askama_parser` | 0.13.0 | MIT OR Apache-2.0 | [github.com/askama-rs/askama](https://github.com/askama-rs/askama) |
| `async-compat` | 0.2.6 | Apache-2.0 OR MIT | [github.com/smol-rs/async-compat](https://github.com/smol-rs/async-compat) |
| `async-compression` | 0.4.51 | MIT OR Apache-2.0 | [github.com/Nullus157/async-compression](https://github.com/Nullus157/async-compression) |
| `async-trait` | 0.1.92 | MIT OR Apache-2.0 | [github.com/dtolnay/async-trait](https://github.com/dtolnay/async-trait) |
| `atomic-waker` | 1.1.2 | Apache-2.0 OR MIT | [github.com/smol-rs/atomic-waker](https://github.com/smol-rs/atomic-waker) |
| `autocfg` | 1.5.1 | Apache-2.0 OR MIT | [github.com/cuviper/autocfg](https://github.com/cuviper/autocfg) |
| `base64` | 0.22.1 | MIT OR Apache-2.0 | [github.com/marshallpierce/rust-base64](https://github.com/marshallpierce/rust-base64) |
| `base64` | 0.23.1 | MIT OR Apache-2.0 | [github.com/marshallpierce/rust-base64](https://github.com/marshallpierce/rust-base64) |
| `base64ct` | 1.8.3 | Apache-2.0 OR MIT | [github.com/RustCrypto/formats](https://github.com/RustCrypto/formats) |
| `basic-toml` | 0.1.10 | MIT OR Apache-2.0 | [github.com/dtolnay/basic-toml](https://github.com/dtolnay/basic-toml) |
| `bitflags` | 2.13.2 | MIT OR Apache-2.0 | [github.com/bitflags/bitflags](https://github.com/bitflags/bitflags) |
| `block-buffer` | 0.10.4 | MIT OR Apache-2.0 | [github.com/RustCrypto/utils](https://github.com/RustCrypto/utils) |
| `brotli` | 9.1.1 | BSD-3-Clause AND MIT | [github.com/dropbox/rust-brotli](https://github.com/dropbox/rust-brotli) |
| `brotli-decompressor` | 6.2.0 | BSD-3-Clause/MIT | [github.com/dropbox/rust-brotli-decompressor](https://github.com/dropbox/rust-brotli-decompressor) |
| `bumpalo` | 3.20.3 | MIT OR Apache-2.0 | [github.com/fitzgen/bumpalo](https://github.com/fitzgen/bumpalo) |
| `bytes` | 1.12.1 | MIT | [github.com/tokio-rs/bytes](https://github.com/tokio-rs/bytes) |
| `camino` | 1.2.6 | MIT OR Apache-2.0 | [github.com/camino-rs/camino](https://github.com/camino-rs/camino) |
| `cargo-platform` | 0.1.9 | MIT OR Apache-2.0 | [github.com/rust-lang/cargo](https://github.com/rust-lang/cargo) |
| `cargo_metadata` | 0.19.2 | MIT | [github.com/oli-obk/cargo_metadata](https://github.com/oli-obk/cargo_metadata) |
| `cc` | 1.10.0 | MIT OR Apache-2.0 | [github.com/rust-lang/cc-rs](https://github.com/rust-lang/cc-rs) |
| `cfg-if` | 1.0.5 | MIT OR Apache-2.0 | [github.com/rust-lang/cfg-if](https://github.com/rust-lang/cfg-if) |
| `cfg_aliases` | 0.2.2 | MIT | [github.com/katharostech/cfg_aliases](https://github.com/katharostech/cfg_aliases) |
| `chacha20` | 0.10.2 | MIT OR Apache-2.0 | [github.com/RustCrypto/stream-ciphers](https://github.com/RustCrypto/stream-ciphers) |
| `compression-codecs` | 0.4.46 | MIT OR Apache-2.0 | [github.com/Nullus157/async-compression](https://github.com/Nullus157/async-compression) |
| `compression-core` | 0.4.34 | MIT OR Apache-2.0 | [github.com/Nullus157/async-compression](https://github.com/Nullus157/async-compression) |
| `const-oid` | 0.9.6 | Apache-2.0 OR MIT | [github.com/RustCrypto/formats/tree/master/const-oid](https://github.com/RustCrypto/formats/tree/master/const-oid) |
| `cpufeatures` | 0.2.17 | MIT OR Apache-2.0 | [github.com/RustCrypto/utils](https://github.com/RustCrypto/utils) |
| `cpufeatures` | 0.3.1 | MIT OR Apache-2.0 | [github.com/RustCrypto/utils](https://github.com/RustCrypto/utils) |
| `crc32fast` | 1.5.3 | MIT OR Apache-2.0 | [github.com/srijs/rust-crc32fast](https://github.com/srijs/rust-crc32fast) |
| `crossbeam-utils` | 0.8.23 | MIT OR Apache-2.0 | [github.com/crossbeam-rs/crossbeam](https://github.com/crossbeam-rs/crossbeam) |
| `crypto-common` | 0.1.7 | MIT OR Apache-2.0 | [github.com/RustCrypto/traits](https://github.com/RustCrypto/traits) |
| `data-encoding` | 2.11.1 | MIT | [github.com/ia0/data-encoding](https://github.com/ia0/data-encoding) |
| `der` | 0.7.10 | Apache-2.0 OR MIT | [github.com/RustCrypto/formats/tree/master/der](https://github.com/RustCrypto/formats/tree/master/der) |
| `derive_arbitrary` | 1.5.0 | MIT OR Apache-2.0 | [github.com/rust-fuzz/arbitrary](https://github.com/rust-fuzz/arbitrary) |
| `digest` | 0.10.7 | MIT OR Apache-2.0 | [github.com/RustCrypto/traits](https://github.com/RustCrypto/traits) |
| `displaydoc` | 0.2.7 | MIT OR Apache-2.0 | [github.com/yaahc/displaydoc](https://github.com/yaahc/displaydoc) |
| `either` | 1.19.0 | MIT OR Apache-2.0 | [github.com/rayon-rs/either](https://github.com/rayon-rs/either) |
| `equivalent` | 1.0.3 | Apache-2.0 OR MIT | [github.com/indexmap-rs/equivalent](https://github.com/indexmap-rs/equivalent) |
| `errno` | 0.3.14 | MIT OR Apache-2.0 | [github.com/lambda-fairy/rust-errno](https://github.com/lambda-fairy/rust-errno) |
| `fallible-iterator` | 0.3.0 | MIT/Apache-2.0 | [github.com/sfackler/rust-fallible-iterator](https://github.com/sfackler/rust-fallible-iterator) |
| `fallible-streaming-iterator` | 0.1.9 | MIT/Apache-2.0 | [github.com/sfackler/fallible-streaming-iterator](https://github.com/sfackler/fallible-streaming-iterator) |
| `fastrand` | 2.5.0 | Apache-2.0 OR MIT | [github.com/smol-rs/fastrand](https://github.com/smol-rs/fastrand) |
| `find-msvc-tools` | 0.1.15 | MIT OR Apache-2.0 | [github.com/rust-lang/cc-rs](https://github.com/rust-lang/cc-rs) |
| `flate2` | 1.1.10 | MIT OR Apache-2.0 | [github.com/rust-lang/flate2-rs](https://github.com/rust-lang/flate2-rs) |
| `form_urlencoded` | 1.2.2 | MIT OR Apache-2.0 | [github.com/servo/rust-url](https://github.com/servo/rust-url) |
| `fs-err` | 2.11.0 | MIT/Apache-2.0 | [github.com/andrewhickman/fs-err](https://github.com/andrewhickman/fs-err) |
| `futures-channel` | 0.3.34 | MIT OR Apache-2.0 | [github.com/rust-lang/futures-rs](https://github.com/rust-lang/futures-rs) |
| `futures-core` | 0.3.34 | MIT OR Apache-2.0 | [github.com/rust-lang/futures-rs](https://github.com/rust-lang/futures-rs) |
| `futures-io` | 0.3.34 | MIT OR Apache-2.0 | [github.com/rust-lang/futures-rs](https://github.com/rust-lang/futures-rs) |
| `futures-sink` | 0.3.34 | MIT OR Apache-2.0 | [github.com/rust-lang/futures-rs](https://github.com/rust-lang/futures-rs) |
| `futures-task` | 0.3.34 | MIT OR Apache-2.0 | [github.com/rust-lang/futures-rs](https://github.com/rust-lang/futures-rs) |
| `futures-util` | 0.3.34 | MIT OR Apache-2.0 | [github.com/rust-lang/futures-rs](https://github.com/rust-lang/futures-rs) |
| `generic-array` | 0.14.7 | MIT | [github.com/fizyk20/generic-array.git](https://github.com/fizyk20/generic-array.git) |
| `getrandom` | 0.2.17 | MIT OR Apache-2.0 | [github.com/rust-random/getrandom](https://github.com/rust-random/getrandom) |
| `getrandom` | 0.3.4 | MIT OR Apache-2.0 | [github.com/rust-random/getrandom](https://github.com/rust-random/getrandom) |
| `getrandom` | 0.4.3 | MIT OR Apache-2.0 | [github.com/rust-random/getrandom](https://github.com/rust-random/getrandom) |
| `glob` | 0.3.4 | MIT OR Apache-2.0 | [github.com/rust-lang/glob](https://github.com/rust-lang/glob) |
| `goblin` | 0.8.2 | MIT | [github.com/m4b/goblin](https://github.com/m4b/goblin) |
| `hashbrown` | 0.14.5 | MIT OR Apache-2.0 | [github.com/rust-lang/hashbrown](https://github.com/rust-lang/hashbrown) |
| `hashbrown` | 0.17.1 | MIT OR Apache-2.0 | [github.com/rust-lang/hashbrown](https://github.com/rust-lang/hashbrown) |
| `hashlink` | 0.9.1 | MIT OR Apache-2.0 | [github.com/kyren/hashlink](https://github.com/kyren/hashlink) |
| `heck` | 0.5.0 | MIT OR Apache-2.0 | [github.com/withoutboats/heck](https://github.com/withoutboats/heck) |
| `http` | 1.5.0 | MIT OR Apache-2.0 | [github.com/hyperium/http](https://github.com/hyperium/http) |
| `http-body` | 1.1.0 | MIT | [github.com/hyperium/http-body](https://github.com/hyperium/http-body) |
| `http-body-util` | 0.1.5 | MIT | [github.com/hyperium/http-body](https://github.com/hyperium/http-body) |
| `httparse` | 1.10.1 | MIT OR Apache-2.0 | [github.com/seanmonstar/httparse](https://github.com/seanmonstar/httparse) |
| `hyper` | 1.12.0 | MIT | [github.com/hyperium/hyper](https://github.com/hyperium/hyper) |
| `hyper-rustls` | 0.27.10 | Apache-2.0 OR ISC OR MIT | [github.com/rustls/hyper-rustls](https://github.com/rustls/hyper-rustls) |
| `hyper-util` | 0.1.21 | MIT | [github.com/hyperium/hyper-util](https://github.com/hyperium/hyper-util) |
| `icu_collections` | 2.4.0 | Unicode-3.0 | [github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| `icu_locale_core` | 2.4.0 | Unicode-3.0 | [github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| `icu_normalizer` | 2.4.0 | Unicode-3.0 | [github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| `icu_normalizer_data` | 2.4.0 | Unicode-3.0 | [github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| `icu_properties` | 2.4.0 | Unicode-3.0 | [github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| `icu_properties_data` | 2.4.0 | Unicode-3.0 | [github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| `icu_provider` | 2.4.0 | Unicode-3.0 | [github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| `idna` | 1.1.0 | MIT OR Apache-2.0 | [github.com/servo/rust-url](https://github.com/servo/rust-url/) |
| `idna_adapter` | 1.2.2 | Apache-2.0 OR MIT | [github.com/hsivonen/idna_adapter](https://github.com/hsivonen/idna_adapter) |
| `indexmap` | 2.14.2 | Apache-2.0 OR MIT | [github.com/indexmap-rs/indexmap](https://github.com/indexmap-rs/indexmap) |
| `ipnet` | 2.12.2 | MIT OR Apache-2.0 | [github.com/krisprice/ipnet](https://github.com/krisprice/ipnet) |
| `itertools` | 0.14.0 | MIT OR Apache-2.0 | [github.com/rust-itertools/itertools](https://github.com/rust-itertools/itertools) |
| `itoa` | 1.0.18 | MIT OR Apache-2.0 | [github.com/dtolnay/itoa](https://github.com/dtolnay/itoa) |
| `js-sys` | 0.3.107 | MIT OR Apache-2.0 | [github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/js-sys](https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/js-sys) |
| `lazy_static` | 1.5.1 | MIT OR Apache-2.0 | [github.com/rust-lang-nursery/lazy-static.rs](https://github.com/rust-lang-nursery/lazy-static.rs) |
| `libc` | 0.2.191 | MIT OR Apache-2.0 | [github.com/rust-lang/libc](https://github.com/rust-lang/libc) |
| `libm` | 0.2.16 | MIT | [github.com/rust-lang/compiler-builtins](https://github.com/rust-lang/compiler-builtins) |
| `libsqlite3-sys` | 0.30.1 | MIT | [github.com/rusqlite/rusqlite](https://github.com/rusqlite/rusqlite) |
| `linux-raw-sys` | 0.12.1 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | [github.com/sunfishcode/linux-raw-sys](https://github.com/sunfishcode/linux-raw-sys) |
| `litemap` | 0.8.3 | Unicode-3.0 | [github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| `log` | 0.4.34 | MIT OR Apache-2.0 | [github.com/rust-lang/log](https://github.com/rust-lang/log) |
| `lru-slab` | 0.1.3 | MIT OR Apache-2.0 OR Zlib | [github.com/Ralith/lru-slab](https://github.com/Ralith/lru-slab) |
| `md-5` | 0.10.6 | MIT OR Apache-2.0 | [github.com/RustCrypto/hashes](https://github.com/RustCrypto/hashes) |
| `memchr` | 2.8.3 | Unlicense OR MIT | [github.com/BurntSushi/memchr](https://github.com/BurntSushi/memchr) |
| `mime` | 0.3.17 | MIT OR Apache-2.0 | [github.com/hyperium/mime](https://github.com/hyperium/mime) |
| `mime_guess` | 2.0.5 | MIT | [github.com/abonander/mime_guess](https://github.com/abonander/mime_guess) |
| `minimal-lexical` | 0.2.1 | MIT/Apache-2.0 | [github.com/Alexhuszagh/minimal-lexical](https://github.com/Alexhuszagh/minimal-lexical) |
| `miniz_oxide` | 0.9.1 | MIT OR Zlib OR Apache-2.0 | [github.com/Frommi/miniz_oxide/tree/master/miniz_oxide](https://github.com/Frommi/miniz_oxide/tree/master/miniz_oxide) |
| `mio` | 1.2.4 | MIT | [github.com/tokio-rs/mio](https://github.com/tokio-rs/mio) |
| `nom` | 7.1.3 | MIT | [github.com/Geal/nom](https://github.com/Geal/nom) |
| `num-bigint-dig` | 0.8.6 | MIT/Apache-2.0 | [github.com/dignifiedquire/num-bigint](https://github.com/dignifiedquire/num-bigint) |
| `num-integer` | 0.1.47 | MIT OR Apache-2.0 | [github.com/rust-num/num-integer](https://github.com/rust-num/num-integer) |
| `num-iter` | 0.1.46 | MIT OR Apache-2.0 | [github.com/rust-num/num-iter](https://github.com/rust-num/num-iter) |
| `num-traits` | 0.2.19 | MIT OR Apache-2.0 | [github.com/rust-num/num-traits](https://github.com/rust-num/num-traits) |
| `once_cell` | 1.21.4 | MIT OR Apache-2.0 | [github.com/matklad/once_cell](https://github.com/matklad/once_cell) |
| `pem-rfc7468` | 0.7.0 | Apache-2.0 OR MIT | [github.com/RustCrypto/formats/tree/master/pem-rfc7468](https://github.com/RustCrypto/formats/tree/master/pem-rfc7468) |
| `percent-encoding` | 2.3.2 | MIT OR Apache-2.0 | [github.com/servo/rust-url](https://github.com/servo/rust-url/) |
| `pin-project-lite` | 0.2.17 | Apache-2.0 OR MIT | [github.com/taiki-e/pin-project-lite](https://github.com/taiki-e/pin-project-lite) |
| `pkcs1` | 0.7.5 | Apache-2.0 OR MIT | [github.com/RustCrypto/formats/tree/master/pkcs1](https://github.com/RustCrypto/formats/tree/master/pkcs1) |
| `pkcs8` | 0.10.2 | Apache-2.0 OR MIT | [github.com/RustCrypto/formats/tree/master/pkcs8](https://github.com/RustCrypto/formats/tree/master/pkcs8) |
| `pkg-config` | 0.3.34 | MIT OR Apache-2.0 | [github.com/rust-lang/pkg-config-rs](https://github.com/rust-lang/pkg-config-rs) |
| `plain` | 0.2.3 | MIT/Apache-2.0 | [github.com/randomites/plain](https://github.com/randomites/plain) |
| `potential_utf` | 0.1.6 | Unicode-3.0 | [github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| `ppv-lite86` | 0.2.21 | MIT OR Apache-2.0 | [github.com/cryptocorrosion/cryptocorrosion](https://github.com/cryptocorrosion/cryptocorrosion) |
| `proc-macro2` | 1.0.107 | MIT OR Apache-2.0 | [github.com/dtolnay/proc-macro2](https://github.com/dtolnay/proc-macro2) |
| `prost` | 0.13.5 | Apache-2.0 | [github.com/tokio-rs/prost](https://github.com/tokio-rs/prost) |
| `prost-derive` | 0.13.5 | Apache-2.0 | [github.com/tokio-rs/prost](https://github.com/tokio-rs/prost) |
| `qrcode` | 0.14.1 | MIT OR Apache-2.0 | [github.com/kennytm/qrcode-rust](https://github.com/kennytm/qrcode-rust) |
| `quinn` | 0.11.12 | MIT OR Apache-2.0 | [github.com/quinn-rs/quinn](https://github.com/quinn-rs/quinn) |
| `quinn-proto` | 0.11.19 | MIT OR Apache-2.0 | [github.com/quinn-rs/quinn](https://github.com/quinn-rs/quinn) |
| `quinn-udp` | 0.5.16 | MIT OR Apache-2.0 | [github.com/quinn-rs/quinn](https://github.com/quinn-rs/quinn) |
| `quote` | 1.0.47 | MIT OR Apache-2.0 | [github.com/dtolnay/quote](https://github.com/dtolnay/quote) |
| `r-efi` | 5.3.0 | MIT OR Apache-2.0 OR LGPL-2.1-or-later | [github.com/r-efi/r-efi](https://github.com/r-efi/r-efi) |
| `r-efi` | 6.0.0 | MIT OR Apache-2.0 OR LGPL-2.1-or-later | [github.com/r-efi/r-efi](https://github.com/r-efi/r-efi) |
| `rand` | 0.10.3 | MIT OR Apache-2.0 | [github.com/rust-random/rand](https://github.com/rust-random/rand) |
| `rand` | 0.8.8 | MIT OR Apache-2.0 | [github.com/rust-random/rand](https://github.com/rust-random/rand) |
| `rand` | 0.9.5 | MIT OR Apache-2.0 | [github.com/rust-random/rand](https://github.com/rust-random/rand) |
| `rand_chacha` | 0.3.1 | MIT OR Apache-2.0 | [github.com/rust-random/rand](https://github.com/rust-random/rand) |
| `rand_chacha` | 0.9.0 | MIT OR Apache-2.0 | [github.com/rust-random/rand](https://github.com/rust-random/rand) |
| `rand_core` | 0.10.1 | MIT OR Apache-2.0 | [github.com/rust-random/rand_core](https://github.com/rust-random/rand_core) |
| `rand_core` | 0.6.4 | MIT OR Apache-2.0 | [github.com/rust-random/rand](https://github.com/rust-random/rand) |
| `rand_core` | 0.9.5 | MIT OR Apache-2.0 | [github.com/rust-random/rand](https://github.com/rust-random/rand) |
| `rand_pcg` | 0.10.2 | MIT OR Apache-2.0 | [github.com/rust-random/rngs](https://github.com/rust-random/rngs) |
| `regex` | 1.13.1 | MIT OR Apache-2.0 | [github.com/rust-lang/regex](https://github.com/rust-lang/regex) |
| `regex-automata` | 0.4.18 | MIT OR Apache-2.0 | [github.com/rust-lang/regex](https://github.com/rust-lang/regex) |
| `regex-syntax` | 0.8.11 | MIT OR Apache-2.0 | [github.com/rust-lang/regex](https://github.com/rust-lang/regex) |
| `reqwest` | 0.12.28 | MIT OR Apache-2.0 | [github.com/seanmonstar/reqwest](https://github.com/seanmonstar/reqwest) |
| `ring` | 0.17.14 | Apache-2.0 AND ISC | [github.com/briansmith/ring](https://github.com/briansmith/ring) |
| `rsa` | 0.9.10 | MIT OR Apache-2.0 | [github.com/RustCrypto/RSA](https://github.com/RustCrypto/RSA) |
| `rusqlite` | 0.32.1 | MIT | [github.com/rusqlite/rusqlite](https://github.com/rusqlite/rusqlite) |
| `rustc-hash` | 2.1.3 | Apache-2.0 OR MIT | [github.com/rust-lang/rustc-hash](https://github.com/rust-lang/rustc-hash) |
| `rustix` | 1.1.6 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | [github.com/bytecodealliance/rustix](https://github.com/bytecodealliance/rustix) |
| `rustls` | 0.23.46 | Apache-2.0 OR ISC OR MIT | [github.com/rustls/rustls](https://github.com/rustls/rustls) |
| `rustls-pki-types` | 1.15.1 | MIT OR Apache-2.0 | [github.com/rustls/pki-types](https://github.com/rustls/pki-types) |
| `rustls-webpki` | 0.103.16 | ISC | [github.com/rustls/webpki](https://github.com/rustls/webpki) |
| `rustversion` | 1.0.23 | MIT OR Apache-2.0 | [github.com/dtolnay/rustversion](https://github.com/dtolnay/rustversion) |
| `ryu` | 1.0.23 | Apache-2.0 OR BSL-1.0 | [github.com/dtolnay/ryu](https://github.com/dtolnay/ryu) |
| `scroll` | 0.12.0 | MIT | [github.com/m4b/scroll](https://github.com/m4b/scroll) |
| `scroll_derive` | 0.12.1 | MIT | [github.com/m4b/scroll](https://github.com/m4b/scroll) |
| `semver` | 1.0.28 | MIT OR Apache-2.0 | [github.com/dtolnay/semver](https://github.com/dtolnay/semver) |
| `serde` | 1.0.229 | MIT OR Apache-2.0 | [github.com/serde-rs/serde](https://github.com/serde-rs/serde) |
| `serde_core` | 1.0.229 | MIT OR Apache-2.0 | [github.com/serde-rs/serde](https://github.com/serde-rs/serde) |
| `serde_derive` | 1.0.229 | MIT OR Apache-2.0 | [github.com/serde-rs/serde](https://github.com/serde-rs/serde) |
| `serde_json` | 1.0.154 | MIT OR Apache-2.0 | [github.com/serde-rs/json](https://github.com/serde-rs/json) |
| `serde_spanned` | 0.6.9 | MIT OR Apache-2.0 | [github.com/toml-rs/toml](https://github.com/toml-rs/toml) |
| `serde_urlencoded` | 0.7.1 | MIT/Apache-2.0 | [github.com/nox/serde_urlencoded](https://github.com/nox/serde_urlencoded) |
| `sha1` | 0.10.7 | MIT OR Apache-2.0 | [github.com/RustCrypto/hashes](https://github.com/RustCrypto/hashes) |
| `shlex` | 2.0.1 | MIT OR Apache-2.0 | [github.com/comex/rust-shlex](https://github.com/comex/rust-shlex) |
| `signature` | 2.2.0 | Apache-2.0 OR MIT | [github.com/RustCrypto/traits/tree/master/signature](https://github.com/RustCrypto/traits/tree/master/signature) |
| `simd-adler32` | 0.3.10 | MIT | [github.com/mcountryman/simd-adler32](https://github.com/mcountryman/simd-adler32) |
| `siphasher` | 0.3.11 | MIT/Apache-2.0 | [github.com/jedisct1/rust-siphash](https://github.com/jedisct1/rust-siphash) |
| `slab` | 0.4.12 | MIT | [github.com/tokio-rs/slab](https://github.com/tokio-rs/slab) |
| `smallvec` | 1.16.4 | MIT OR Apache-2.0 | [github.com/servo/rust-smallvec](https://github.com/servo/rust-smallvec) |
| `smawk` | 0.3.3 | MIT | [github.com/mgeisler/smawk](https://github.com/mgeisler/smawk) |
| `socket2` | 0.6.5 | MIT OR Apache-2.0 | [github.com/rust-lang/socket2](https://github.com/rust-lang/socket2) |
| `spin` | 0.9.9 | MIT | [github.com/mvdnes/spin-rs.git](https://github.com/mvdnes/spin-rs.git) |
| `spki` | 0.7.3 | Apache-2.0 OR MIT | [github.com/RustCrypto/formats/tree/master/spki](https://github.com/RustCrypto/formats/tree/master/spki) |
| `stable_deref_trait` | 1.2.1 | MIT OR Apache-2.0 | [github.com/storyyeller/stable_deref_trait](https://github.com/storyyeller/stable_deref_trait) |
| `static_assertions` | 1.1.0 | MIT OR Apache-2.0 | [github.com/nvzqz/static-assertions-rs](https://github.com/nvzqz/static-assertions-rs) |
| `subtle` | 2.6.1 | BSD-3-Clause | [github.com/dalek-cryptography/subtle](https://github.com/dalek-cryptography/subtle) |
| `syn` | 2.0.119 | MIT OR Apache-2.0 | [github.com/dtolnay/syn](https://github.com/dtolnay/syn) |
| `syn` | 3.0.9 | MIT OR Apache-2.0 | [github.com/dtolnay/syn](https://github.com/dtolnay/syn) |
| `sync_wrapper` | 1.0.2 | Apache-2.0 | [github.com/Actyx/sync_wrapper](https://github.com/Actyx/sync_wrapper) |
| `synstructure` | 0.14.0 | MIT | [github.com/mystor/synstructure](https://github.com/mystor/synstructure) |
| `tempfile` | 3.27.0 | MIT OR Apache-2.0 | [github.com/Stebalien/tempfile](https://github.com/Stebalien/tempfile) |
| `textwrap` | 0.16.4 | MIT | [github.com/mgeisler/textwrap](https://github.com/mgeisler/textwrap) |
| `thiserror` | 2.0.21 | MIT OR Apache-2.0 | [github.com/dtolnay/thiserror](https://github.com/dtolnay/thiserror) |
| `thiserror-impl` | 2.0.21 | MIT OR Apache-2.0 | [github.com/dtolnay/thiserror](https://github.com/dtolnay/thiserror) |
| `tinystr` | 0.8.5 | Unicode-3.0 | [github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| `tinyvec` | 1.13.3 | Zlib OR Apache-2.0 OR MIT | [github.com/Lokathor/tinyvec](https://github.com/Lokathor/tinyvec) |
| `tokio` | 1.54.1 | MIT | [github.com/tokio-rs/tokio](https://github.com/tokio-rs/tokio) |
| `tokio-macros` | 2.7.2 | MIT | [github.com/tokio-rs/tokio](https://github.com/tokio-rs/tokio) |
| `tokio-rustls` | 0.26.6 | MIT OR Apache-2.0 | [github.com/rustls/tokio-rustls](https://github.com/rustls/tokio-rustls) |
| `tokio-tungstenite` | 0.26.2 | MIT | [github.com/snapview/tokio-tungstenite](https://github.com/snapview/tokio-tungstenite) |
| `tokio-util` | 0.7.20 | MIT | [github.com/tokio-rs/tokio](https://github.com/tokio-rs/tokio) |
| `toml` | 0.5.11 | MIT/Apache-2.0 | [github.com/toml-rs/toml](https://github.com/toml-rs/toml) |
| `toml` | 0.8.23 | MIT OR Apache-2.0 | [github.com/toml-rs/toml](https://github.com/toml-rs/toml) |
| `toml_datetime` | 0.6.11 | MIT OR Apache-2.0 | [github.com/toml-rs/toml](https://github.com/toml-rs/toml) |
| `toml_edit` | 0.22.27 | MIT OR Apache-2.0 | [github.com/toml-rs/toml](https://github.com/toml-rs/toml) |
| `toml_write` | 0.1.2 | MIT OR Apache-2.0 | [github.com/toml-rs/toml](https://github.com/toml-rs/toml) |
| `tower` | 0.5.3 | MIT | [github.com/tower-rs/tower](https://github.com/tower-rs/tower) |
| `tower-http` | 0.6.11 | MIT | [github.com/tower-rs/tower-http](https://github.com/tower-rs/tower-http) |
| `tower-layer` | 0.3.3 | MIT | [github.com/tower-rs/tower](https://github.com/tower-rs/tower) |
| `tower-service` | 0.3.3 | MIT | [github.com/tower-rs/tower](https://github.com/tower-rs/tower) |
| `tracing` | 0.1.44 | MIT | [github.com/tokio-rs/tracing](https://github.com/tokio-rs/tracing) |
| `tracing-core` | 0.1.36 | MIT | [github.com/tokio-rs/tracing](https://github.com/tokio-rs/tracing) |
| `try-lock` | 0.2.5 | MIT | [github.com/seanmonstar/try-lock](https://github.com/seanmonstar/try-lock) |
| `tungstenite` | 0.26.2 | MIT OR Apache-2.0 | [github.com/snapview/tungstenite-rs](https://github.com/snapview/tungstenite-rs) |
| `typenum` | 1.20.1 | MIT OR Apache-2.0 | [github.com/paholg/typenum](https://github.com/paholg/typenum) |
| `unicase` | 2.10.0 | MIT OR Apache-2.0 | [github.com/seanmonstar/unicase](https://github.com/seanmonstar/unicase) |
| `unicode-ident` | 1.0.27 | (MIT OR Apache-2.0) AND Unicode-3.0 | [github.com/dtolnay/unicode-ident](https://github.com/dtolnay/unicode-ident) |
| `unicode-width` | 0.2.3 | MIT OR Apache-2.0 | [github.com/unicode-rs/unicode-width](https://github.com/unicode-rs/unicode-width) |
| `uniffi` | 0.29.5 | MPL-2.0 | [github.com/mozilla/uniffi-rs](https://github.com/mozilla/uniffi-rs) |
| `uniffi_bindgen` | 0.29.5 | MPL-2.0 | [github.com/mozilla/uniffi-rs](https://github.com/mozilla/uniffi-rs) |
| `uniffi_core` | 0.29.5 | MPL-2.0 | [github.com/mozilla/uniffi-rs](https://github.com/mozilla/uniffi-rs) |
| `uniffi_internal_macros` | 0.29.5 | MPL-2.0 | [github.com/mozilla/uniffi-rs](https://github.com/mozilla/uniffi-rs) |
| `uniffi_macros` | 0.29.5 | MPL-2.0 | [github.com/mozilla/uniffi-rs](https://github.com/mozilla/uniffi-rs) |
| `uniffi_meta` | 0.29.5 | MPL-2.0 | [github.com/mozilla/uniffi-rs](https://github.com/mozilla/uniffi-rs) |
| `uniffi_pipeline` | 0.29.5 | MPL-2.0 | [github.com/mozilla/uniffi-rs](https://github.com/mozilla/uniffi-rs) |
| `uniffi_udl` | 0.29.5 | MPL-2.0 | [github.com/mozilla/uniffi-rs](https://github.com/mozilla/uniffi-rs) |
| `untrusted` | 0.9.0 | ISC | [github.com/briansmith/untrusted](https://github.com/briansmith/untrusted) |
| `url` | 2.5.8 | MIT OR Apache-2.0 | [github.com/servo/rust-url](https://github.com/servo/rust-url) |
| `utf-8` | 0.7.6 | MIT OR Apache-2.0 | [github.com/SimonSapin/rust-utf8](https://github.com/SimonSapin/rust-utf8) |
| `utf8_iter` | 1.1.0 | Apache-2.0 OR MIT | [github.com/hsivonen/utf8_iter](https://github.com/hsivonen/utf8_iter) |
| `utf_types` | 1.0.0 | Apache-2.0 OR MIT | [github.com/hsivonen/utf_types](https://github.com/hsivonen/utf_types) |
| `vcpkg` | 0.2.15 | MIT/Apache-2.0 | [github.com/mcgoo/vcpkg-rs](https://github.com/mcgoo/vcpkg-rs) |
| `version_check` | 0.9.5 | MIT/Apache-2.0 | [github.com/SergioBenitez/version_check](https://github.com/SergioBenitez/version_check) |
| `want` | 0.3.2 | MIT | [github.com/seanmonstar/want](https://github.com/seanmonstar/want) |
| `wasi` | 0.11.1+wasi-snapshot-preview1 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | [github.com/bytecodealliance/wasi](https://github.com/bytecodealliance/wasi) |
| `wasip2` | 1.0.4+wasi-0.2.12 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | [github.com/bytecodealliance/wasi-rs](https://github.com/bytecodealliance/wasi-rs) |
| `wasm-bindgen` | 0.2.130 | MIT OR Apache-2.0 | [github.com/wasm-bindgen/wasm-bindgen](https://github.com/wasm-bindgen/wasm-bindgen) |
| `wasm-bindgen-futures` | 0.4.80 | MIT OR Apache-2.0 | [github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/futures](https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/futures) |
| `wasm-bindgen-macro` | 0.2.130 | MIT OR Apache-2.0 | [github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/macro](https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/macro) |
| `wasm-bindgen-macro-support` | 0.2.130 | MIT OR Apache-2.0 | [github.com/wasm-bindgen/wasm-bindgen/tree/main/crates/macro-support](https://github.com/wasm-bindgen/wasm-bindgen/tree/main/crates/macro-support) |
| `wasm-bindgen-shared` | 0.2.130 | MIT OR Apache-2.0 | [github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/shared](https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/shared) |
| `web-sys` | 0.3.107 | MIT OR Apache-2.0 | [github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/web-sys](https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/web-sys) |
| `web-time` | 1.1.0 | MIT OR Apache-2.0 | [github.com/daxpedda/web-time](https://github.com/daxpedda/web-time) |
| `webpki-roots` | 0.26.11 | CDLA-Permissive-2.0 | [github.com/rustls/webpki-roots](https://github.com/rustls/webpki-roots) |
| `webpki-roots` | 1.0.11 | CDLA-Permissive-2.0 | [github.com/rustls/webpki-roots](https://github.com/rustls/webpki-roots) |
| `weedle2` | 5.0.0 | MIT | [github.com/mozilla/uniffi-rs](https://github.com/mozilla/uniffi-rs) |
| `windows-collections` | 0.100.0 | MIT OR Apache-2.0 | [github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows-core` | 0.100.0 | MIT OR Apache-2.0 | [github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows-future` | 0.100.0 | MIT OR Apache-2.0 | [github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows-implement` | 0.100.0 | MIT OR Apache-2.0 | [github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows-interface` | 0.100.0 | MIT OR Apache-2.0 | [github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows-link` | 0.100.0 | MIT OR Apache-2.0 | [github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows-link` | 0.2.1 | MIT OR Apache-2.0 | [github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows-numerics` | 0.100.0 | MIT OR Apache-2.0 | [github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows-reference` | 0.100.0 | MIT OR Apache-2.0 | [github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows-result` | 0.100.0 | MIT OR Apache-2.0 | [github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows-strings` | 0.100.0 | MIT OR Apache-2.0 | [github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows-sys` | 0.52.0 | MIT OR Apache-2.0 | [github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows-sys` | 0.61.2 | MIT OR Apache-2.0 | [github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows-targets` | 0.52.6 | MIT OR Apache-2.0 | [github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows-threading` | 0.100.0 | MIT OR Apache-2.0 | [github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows-time` | 0.100.0 | MIT OR Apache-2.0 | [github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows_aarch64_gnullvm` | 0.52.6 | MIT OR Apache-2.0 | [github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows_aarch64_msvc` | 0.52.6 | MIT OR Apache-2.0 | [github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows_i686_gnu` | 0.52.6 | MIT OR Apache-2.0 | [github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows_i686_gnullvm` | 0.52.6 | MIT OR Apache-2.0 | [github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows_i686_msvc` | 0.52.6 | MIT OR Apache-2.0 | [github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows_x86_64_gnu` | 0.52.6 | MIT OR Apache-2.0 | [github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows_x86_64_gnullvm` | 0.52.6 | MIT OR Apache-2.0 | [github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows_x86_64_msvc` | 0.52.6 | MIT OR Apache-2.0 | [github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `winnow` | 0.7.15 | MIT | [github.com/winnow-rs/winnow](https://github.com/winnow-rs/winnow) |
| `wit-bindgen` | 0.57.1 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | [github.com/bytecodealliance/wit-bindgen](https://github.com/bytecodealliance/wit-bindgen) |
| `writeable` | 0.6.4 | Unicode-3.0 | [github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| `yoke` | 0.8.4 | Unicode-3.0 | [github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| `yoke-derive` | 0.8.4 | Unicode-3.0 | [github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| `zerocopy` | 0.8.63 | BSD-2-Clause OR Apache-2.0 OR MIT | [github.com/google/zerocopy](https://github.com/google/zerocopy) |
| `zerocopy-derive` | 0.8.63 | BSD-2-Clause OR Apache-2.0 OR MIT | [github.com/google/zerocopy](https://github.com/google/zerocopy) |
| `zerofrom` | 0.1.8 | Unicode-3.0 | [github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| `zerofrom-derive` | 0.1.8 | Unicode-3.0 | [github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| `zeroize` | 1.9.1 | Apache-2.0 OR MIT | [github.com/RustCrypto/utils](https://github.com/RustCrypto/utils) |
| `zerotrie` | 0.2.5 | Unicode-3.0 | [github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| `zerovec` | 0.11.9 | Unicode-3.0 | [github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| `zerovec-derive` | 0.11.7 | Unicode-3.0 | [github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| `zip` | 2.6.1 | MIT | [github.com/zip-rs/zip2.git](https://github.com/zip-rs/zip2.git) |
| `zlib-rs` | 0.6.8 | Zlib | [github.com/trifectatechfoundation/zlib-rs](https://github.com/trifectatechfoundation/zlib-rs) |
| `zmij` | 1.0.23 | MIT | [github.com/dtolnay/zmij](https://github.com/dtolnay/zmij) |
| `zopfli` | 0.8.4 | Apache-2.0 | [github.com/zopfli-rs/zopfli](https://github.com/zopfli-rs/zopfli) |

## Kotlin / Android

| 组件 | 许可 | 上游 |
|---|---|---|
| Kotlin 标准库 | Apache-2.0 | https://github.com/JetBrains/kotlin |
| AndroidX（`androidx.*`） | Apache-2.0 | https://android.googlesource.com/platform/frameworks/support |
| Jetpack Compose（`androidx.compose.*`） | Apache-2.0 | https://android.googlesource.com/platform/frameworks/support |
| Media3 / ExoPlayer | Apache-2.0 | https://github.com/androidx/media |
| OkHttp / Okio | Apache-2.0 | https://github.com/square/okhttp |

## 说明

- **MPL-2.0 组件**：本仓库没有改动它们，用的是发布到 crates.io 的原样版本；如果以后改了，按该许可需要公开被改文件的源码。
- **字体**：界面用系统字体（Windows 的 Segoe UI / Android 的系统无衬线），仓库里没有内置字体文件，因此不涉及字体许可。
- **图标**：全部是用代码或字符画出来的几何图形，没有引入第三方图标集。
- 这些组件各自的著作权归其作者所有；本仓库不对它们主张任何权利。
