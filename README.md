![Plow logo](./assets/logo.svg)

# Plow - Ontology package manager

Plow is package management solution for OWL ontologies, with support for specifying dependencies between packages via [SemVer](https://semver.org/) ranges.

## Getting started - Installation

### CLI

The CLI supports basic commands related to consuming and producing ontologies. It is suitable for both manual and automated workflows (e.g. [metadata linting in CI](https://github.com/field33/ontologies/blob/12ede2b557fde94f6a768e8b65c84929a58c05ce/.github/workflows/lint.yml#L33))

To install, run:

```sh
cargo install plow_cli
```

[Prebuilt binaries are coming soon!](https://github.com/field33/plow/issues/1)

### GUI

The [`plow_gui`](https://crates.io/crates/plow_gui) crate is obsolete and usage of it is discouraged.

A new ontology editor is in the works, and will be released as a separate project.

## Basic usage

### Login with Plow

The tooling currently expects you to be authenticated with the public plow registry ([open issue](https://github.com/field33/plow/issues/11)). Please sign in, creating an account [here](https://plow.pm) and [create a new api token in your account settings](https://registry.field33.com/home#user-tokens).

```sh
plow login <api-token>
```

### Initialize workspace

Create the directory you want to organize your _fields_ (= ontologies) in.

```sh
# Create workspace directory
mkdir example_workspace && cd example_workspace

# Crate your first field (.ttl file), or copy existing fields into the workspace
plow init --field @example_namespace/example_name

# To initialize a workspace run
plow init
```

### Initialize a new _field_ (= ontology)

To create a new _field_ with all the necessary metadata run:

```sh
plow init --field @example_namespace/example_name
```

When run under an initialized workspace, this will create the relevant folder structure in the `fields` directory:

```
├── Plow.toml
└── fields
    └── @example_namespace
        └── example_fieldname.ttl
```

If run outside of a workspace, it will create a new `.ttl` file in the current directory.

```
├── example_fieldname.ttl
```

Running `plow init` without the `--field` flag initializes a new workspace and if run after this, results would look like the most upper example.

### Open a _field_ in protege

If you'd like to open an edit a field in protege, you may use the following command:

```sh
# Continuing with the upper example
plow protege example_fieldname.ttl
```

If you have protege installed in your system and if your field does not have parsing errors, this command will,

- Resolve the dependencies in your field if there are some.
- Inject them to your original field as `owl:imports` annotations.
- Make a `protege_workspaces` directory in `~/Documents/plow`, copy your dependencies and hard link your field there.
- The changes to your field in `protege` will reflect to your original field permanently.

### Submit a _field_ to the registry

To prepare for submitting a new _field_ run the following command:

```shell
plow submit <path-to-your-field> --dry-run
```

If all checks pass you can omit the `--dry-run` flag and submit your _field_ by running:

```shell
# Public submission
plow submit <path-to-your-field>

# Private submission
plow submit --private <path-to-your-field>
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

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
