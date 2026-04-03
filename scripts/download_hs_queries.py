#!/usr/bin/env python3

import argparse
import subprocess
import textwrap
from pathlib import Path

ROOT_DIR = Path(__file__).absolute().parent.parent
HS_DIR = ROOT_DIR / "languages" / "haskell"


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("commit", help="Commit in tek/tree-sitter")
    args = parser.parse_args()

    gh_base_url = (
        f"https://raw.githubusercontent.com/tek/tree-sitter-haskell/{args.commit}"
    )
    header = "\n".join(
        [
            "; ------------------------------------------------------------------------------",
            f"; Adapted from {gh_base_url}",
            "; See scripts/download_hs_queries.py",
            ";",
            "",
        ]
    )

    print("Downloading highlights.scm...")
    highlights = curl(f"{gh_base_url}/queries/highlights.scm")
    highlights = update_highlights(highlights)
    (HS_DIR / "highlights.scm").write_text(header + highlights)

    print("Downloading injections.scm...")
    injections = curl(f"{gh_base_url}/queries/injections.scm")
    (HS_DIR / "injections.scm").write_text(header + injections)


def update_highlights(s: str) -> str:
    sig1 = textwrap.dedent("""\
    (decl/signature
      name: (variable) @_name
      type: (type))
    """)
    sig2 = textwrap.dedent("""\
    (decl/signature
      name: (variable) @_name
      type: (type/apply
        constructor: (name) @_type)
      (#eq? @_type "IO"))
    """)
    sig3 = textwrap.dedent("""\
    (decl/signature
      name: (variable) @_name
      type: (quantified_type))
    """)
    return (
        s
        # Zed doesn't have @character
        .replace("@character", "@string")
        # Zed doesn't have @module
        .replace("@module", "@title")
        # @keyword makes these variables look like Haskell syntax keywords
        # instead of just normal built-in functions
        .replace("@keyword.exception", "@constant.builtin")
        .replace("@keyword.debug", "@constant.builtin")
        # @_name is used in some signature highlighting for a local capture,
        # which overrides the relevant highlighting
        # We also need to add some arbitrary suffix to avoid clobbering the
        # scope for (#eq? ...)
        .replace(sig1, sig1.replace("@_name", "@_name @variable.x"))
        .replace(sig2, sig2.replace("@_name", "@_name @function.x"))
        .replace(sig3, sig3.replace("@_name", "@_name @function.x"))
        # @_op is used as a local capture for predicates, but Zed will actually
        # override the @operator capture previously set on (operator) nodes.
        .replace("@_op", "@operator")
        # @spell isn't valid in Zed and overrides the @comment capture. Comment out
        # this line.
        .replace("(comment) @spell", ";(comment) @spell")
    )


def curl(url: str) -> str:
    proc = subprocess.run(
        ["curl", "-sS", url],
        stdout=subprocess.PIPE,
        check=True,
        text=True,
    )
    return proc.stdout


if __name__ == "__main__":
    main()
