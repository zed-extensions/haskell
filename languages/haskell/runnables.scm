; Detect the main function
(declarations
  [
    (function name: (variable) @run) @_
    (bind name: (variable) @run) @_
  ]
  (#eq? @run "main")
  (#set! tag haskell-build)
  (#set! tag haskell-run))
