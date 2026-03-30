(data_type
  "data" @context
  name: (name) @name) @item

(type_synomym ; typo: https://github.com/tree-sitter/tree-sitter-haskell/pull/145
  "type" @context
  name: (name) @name) @item

(newtype
  "newtype" @context
  name: (name) @name) @item

; Only top-level signatures
(declarations
  (signature
    name: (variable) @name
    "::" @context
    type: _ @context) @item)

; Only top-level binds
(declarations
  [
    (bind
      name: (variable) @context)
    (function
      name: (variable) @context)
  ] @item)

(class
  "class" @context
  (name) @name) @item

(instance
  "instance" @context
  name: _ @name
  patterns: _ @context) @item

(foreign_import
  "foreign" @context
  (entity) @name) @item

; Support for BDD-style test suites, e.g. hspec, skeletest
(apply
  function: [
    (variable)
    (qualified
      (variable))
  ] @_name @context
  (#any-of? @_name "describe" "it" "test" "prop")
  argument: [
    (literal
      (string))
    (variable)
  ] @name) @item
