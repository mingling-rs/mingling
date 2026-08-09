<h1 align="center">Mingling CLI</h1>
<p align="center">
    A tool for creating, managing, inspecting, and installing your Mingling programs
</p>

## Install

> [!NOTE]
> It is not recommended to use `cargo install` to install `mingling-cli` from [crates.io](https://crates.io/crates/mingling-cli), as it does not include the initialization script.

`mling` will be released alongside `mingling@0.4` and later versions on [Github](https://github.com/mingling-rs/mingling/releases). You can download the pre-built version from there and install it by loading the `load_mling` script included in the release.

## Generate Project

```bash
# Set the template source
mling cfg tmpl-source https://github.com/mingling-rs/tmpl.git

# Create a template
mling proj-init 0.4@basic
```

Afterward, `mling` will create a `checklist.toml` in your current directory. Please complete and fill it in:

```bash
nano ./checklist.toml
```

Once you've finished writing, continue with:

```bash
# Continue setting up the project
mling proj-init
```

## Install & Manage Projects

`mling` provides a quick way to install and debug your Mingling programs

```bash
# Install your program
mling install

# Enable your package
mling pkg-enable your-cli@0.1.0

# .. Restart your Shell, and you'll be able to access it
your-cli

# When you're done testing, you can disable the package to avoid cluttering your Shell environment
mling pkg-disable your-cli

# Or you can uninstall it directly
mling uninstall your-cli
```

## Linter (In Development)

`mling` provides a linter to check the quality of your code

```bash
# Run the linter
mling lint

# Explain a lint
mling explain <LINT-NAME>
```

### Mingling Linter & Rust Analyzer

You can integrate `mling lint` into RA using `mling ra-lint-clippy` or `mling ra-lint-check`. Simply enable the following in your RA settings:

```jsonc
{
    // use `mling ra-lint-check` override
    "rust-analyzer.check.overrideCommand": ["mling", "ra-lint-check"],
    "rust-analyzer.checkOnSave": true,
}
```

## License

Under MIT OR Apache-2.0
