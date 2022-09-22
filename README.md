![Plow logo](./assets/logo.svg)
# Plow - Ontology package manager

Plow is package management solution for OWL ontologies, with support for specifying dependencies between packages via SemVer ranges.

## Getting started - Installation

### CLI
The CLI supports basic commands related to consuming and producing ontologies. It is suitable for both manual and automated workflows (e.g. [metadata linting in CI](https://github.com/field33/ontologies/blob/12ede2b557fde94f6a768e8b65c84929a58c05ce/.github/workflows/lint.yml#L33))

Coming soon: [open in Protégé](https://github.com/field33/plow/issues/10)

To install, run:

```shell
cargo install plow_cli
```

[Prebuilt binaries are coming soon!](https://github.com/field33/plow/issues/1)

### GUI

Coming soon

> You can install a preview version with limited functionality via `cargo install plow_cli`

[Prebuilt binaries are coming soon!](https://github.com/field33/plow/issues/2)

## Basic usage
### Login with Plow
The tooling currently expects you to be authenticated with the public plow registry ([open issue](https://github.com/field33/plow/issues/11)). Please sign in, creating an account [here](plow.pm) and [create a new User Token in your account settings](https://staging-registry.field33.com/home#user-tokens).


```shell
plow login <YOUR_TOKEN>
```

### Initialize workspace
Create the directory you want to organize your *fields* (= ontologies) in.
```shell
# Create workspace directory
mkdir example_workspace && cd example_workspace

# Workaround: plow init currently expects a .ttl file to be present
touch test.ttl

# Initializes the workspace in your directory
plow init
```


### Initialize a new *field* (= ontology)
To create a new *field* with all the necessary metadata run:
```shell
plow init --field @example_namespace/example_fieldname
```
This will create the relevant folder structure:
```
├── Plow.toml
└── fields
    └── @example_namespace
        └── example_fieldname.ttl
```

### Submit an *field* to the registry
To prepare for submitting a new *field* run the following command:
```shell
plow submit --dry-run fields/@example_namespace/example_fieldname.ttl
```

If all checks pass you can omit the `--dry-run` flag and propperly submit your *field* by running:
```shell
# Public submission
plow submit fields/@example_namespace/example_fieldname.ttl

# Private submission
plow submit --private fields/@example_namespace/example_fieldname.ttl
```

## Repository contents

- [`plow_cli`](./plow_cli) [<img alt="crates.io" src="https://img.shields.io/crates/v/plow_cli.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/plow_cli) - CLI
- [`plow_gui`](./plow_gui) [<img alt="crates.io" src="https://img.shields.io/crates/v/plow_gui.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/plow_gui) - GUI
- [`plow_linter`](./plow_linter) [<img alt="crates.io" src="https://img.shields.io/crates/v/plow_linter.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/plow_linter) - Linter library (included in CLI/GUI)
- [`plow_package_management`](./plow_package_management) [<img alt="crates.io" src="https://img.shields.io/crates/v/plow_package_management.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/plow_package_management) - Core logic of the package management
- [`plow_ontology`](./plow_ontology) [<img alt="crates.io" src="https://img.shields.io/crates/v/plow_ontology.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/plow_ontology) - Representation of an ontology package and related helpers
- [`plow_graphify`](./plow_graphify) [<img alt="crates.io" src="https://img.shields.io/crates/v/plow_graphify.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/plow_graphify) - Bridge to parse ontologies based on [harriet](https://github.com/field33/harriet)
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
@incollection{Plow,
  title = {Plow: A Novel Approach to Interlinking Modular Ontologies Based on Software Package Management},
  doi = {10.3233/ssw220015},
  url = {https://doi.org/10.3233/ssw220015},
  year = {2022},
  month = sep,
  publisher = {{IOS} Press},
  author = {Maximilian Goisser and Daniel Fiebig and Sebastian Wohlrapp and Georg Rehm},
  booktitle = {Towards a Knowledge-Aware {AI}}
}
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
