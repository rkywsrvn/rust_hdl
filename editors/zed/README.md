# VHDL LS for Zed

This extension adds VHDL support to [Zed](https://zed.dev):

- Syntax highlighting, outline, bracket matching, auto-indent and Vim text
  objects, based on [tree-sitter-vhdl](https://github.com/jpt13653903/tree-sitter-vhdl).
- Diagnostics, completion, hover, go to definition, references, rename and
  symbols from the VHDL-LS language server.

## Install

To install from this repository:

1. Install Rust with [rustup](https://rustup.rs). Zed uses it to build the extension.
2. In Zed, run `zed: install dev extension` and select the `editors/zed` directory.

## Language server

The extension uses the first `vhdl_ls` it finds:

1. The path in the `lsp.vhdl_ls.binary.path` setting.
2. A `vhdl_ls` on your `PATH`.
3. The latest [release](https://github.com/VHDL-LS/rust_hdl/releases/latest).
   The extension downloads it automatically. Releases exist for Linux x86_64,
   macOS ARM64 and Windows x86_64.

`vhdl_ls` needs the standard libraries (`vhdl_libraries`) next to the binary.
The release archives contain them. If you build `vhdl_ls` yourself, pass the
library path with `--libraries`:

```json
{
  "lsp": {
    "vhdl_ls": {
      "binary": {
        "path": "/path/to/vhdl_ls",
        "arguments": ["--libraries", "/path/to/rust_hdl/vhdl_libraries"]
      }
    }
  }
}
```

## Configuration

Put a `vhdl_ls.toml` file in the root of your project to map files to
libraries. See the [main README](../../README.md#configuration) for the format.

By default, the server also analyzes files that are not part of the project.
To turn this off:

```json
{
  "lsp": {
    "vhdl_ls": {
      "initialization_options": {
        "nonProjectFiles": "ignore"
      }
    }
  }
}
```

The server also provides semantic tokens. To use them on top of the
tree-sitter highlighting:

```json
{
  "languages": {
    "VHDL": {
      "semantic_tokens": "combined"
    }
  }
}
```
