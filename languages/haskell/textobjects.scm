(comment)+ @comment.around

[
  (data_type)
  (type_synomym) ; typo: https://github.com/tree-sitter/tree-sitter-haskell/pull/145
  (newtype)
] @class.around

(fields
  "{"
  (_)* @class.inside
  "}")

((signature)?
  (function)+) @function.around

(function
  (match
      expression: (_) @function.inside))
