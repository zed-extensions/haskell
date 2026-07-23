; Detect the main function
(declarations
  [
    (function
      name: (variable) @run)
    (bind
      name: (variable) @run)
  ]
  (#eq? @run "main")
  (#set! tag haskell-build)
  (#set! tag haskell-run))

; Detect describe/it test blocks
((apply
  function: (variable) @run
  (#any-of? @run "describe" "it")
  argument: (literal
    (string) @HASKELL_TEST_NAME))
  (#set! tag haskell-test))
