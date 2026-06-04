# Zed Haskell

A [Haskell](https://www.haskell.org/) extension for [Zed](https://zed.dev).

## Development

To develop this extension, see the [Developing Extensions](https://zed.dev/docs/extensions/developing-extensions) section of the Zed docs.

### Update grammar/queries

1. Bump commit in `extension.toml` to a commit in `tree-sitter/tree-sitter-haskell`
2. Run `scripts/download_hs_queries.py` with a commit in `tek/tree-sitter-haskell`

`tek/tree-sitter-haskell` is more up-to-date than `tree-sitter/tree-sitter-haskell`, so ideally, we'd switch to it ([issue](https://github.com/tek/tree-sitter-haskell/issues/11)). However, `tek/tree-sitter-haskell` doesn't commit `parser.c`, which Zed depends on ([issue](https://github.com/zed-industries/zed/discussions/52532)). Furthermore, regenerating `parser.c` and pointing the extension to it causes Zed to freeze ([issue](https://github.com/zed-industries/zed/issues/52535)). So for now, we'll use `tree-sitter/tree-sitter-haskell` for the grammar, but `tek/tree-sitter-haskell` for the queries.
