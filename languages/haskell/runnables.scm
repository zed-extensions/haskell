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
