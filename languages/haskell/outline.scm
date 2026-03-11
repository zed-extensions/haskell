(data_type
  "data" @context
  name: (name) @name) @item

(type_synomym  ; typo: https://github.com/tree-sitter/tree-sitter-haskell/pull/145
  "type" @context
  name: (name) @name) @item

(newtype
  "newtype" @context
  name: (name) @name) @item

(signature
  name: (variable) @name) @item

(class
  "class" @context
  (name) @name) @item

(instance
  "instance" @context
  (name) @name) @item

(foreign_import
  "foreign" @context
  (entity) @name) @item
