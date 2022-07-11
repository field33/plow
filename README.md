![Plow logo](./assets/logo.svg)
# Plow - Ontology package manager

Plow is package management solution for OWL ontologies, with support for specifying dependencies between packages via SemVer ranges.

## Getting started - Installation

### GUI

To install, run:

```shell
cargo install plow_gui
```

[Prebuilt binaries are coming soon!](https://github.com/field33/plow/issues/2)

### CLI

To install, run:

```shell
cargo install plow_cli
```

[Prebuilt binaries are coming soon!](https://github.com/field33/plow/issues/1)

## Repository contents

- [`plow_cli`](./plow_cli) [<img alt="crates.io" src="https://img.shields.io/crates/v/plow_cli.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/plow_cli) - CLI
- [`plow_gui`](./plow_gui) [<img alt="crates.io" src="https://img.shields.io/crates/v/plow_gui.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/plow_gui) - GUI
- [`plow_linter`](./plow_linter) [<img alt="crates.io" src="https://img.shields.io/crates/v/plow_gui.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/plow_gui) - Linter library (included in CLI/GUI)
- [`plow_package_management`](./plow_package_management) [<img alt="crates.io" src="https://img.shields.io/crates/v/plow_gui.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/plow_gui) - Core logic of the package management
- [`plow_ontology`](./plow_ontology) [<img alt="crates.io" src="https://img.shields.io/crates/v/plow_gui.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/plow_gui) - Representation of an ontology package and related helpers
- [`plow_graphify`](./plow_graphify) [<img alt="crates.io" src="https://img.shields.io/crates/v/plow_gui.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/plow_gui) - Bridge to parse ontologies based on [harriet](https://github.com/field33/harriet)
- [`plow_backend_reference`](./plow_backend_reference) - Reference implementation of the registry service

## Reference implementation and [registry.field33.com](http://registry.field33.com)

We provide a reference implementation of the registry service under [`plow_backend_reference`](./plow_backend_reference).
The implementation documents and showcases all the REST API endpoints required for package management,
but some of the functionality is only implemented in a limited fashio.
E.g. it does not persist any data between process restarts, and doesn't include any authentication/authorization, making it unfit for production usage.

For production usage, we provide a hosted registry with a web UI at [registry.field33.com](http://registry.field33.com).
As the underlying codebase is strongly connected to other parts of our products, it is currently not viable for us to maintain
the registry publicly, but that may change in the future.

## Citing

If you use Plow in the context of a published academic piece of work, please consider citing:
```
TODO: Add citation upon publication
```

## Contributing

We are happy about any contributions!

To get started you can take a look at our [Github issues](https://github.com/field33/plow/issues).

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as below, without any additional terms or
conditions.

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
