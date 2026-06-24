; Detect describe/it test blocks
((apply
  function: (variable) @run
  (#any-of? @run "describe" "it")
  argument: (literal (string) @HASKELL_TEST_NAME)) @_haskell-test
  (#set! tag haskell-test))
