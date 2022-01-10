# `nanopre`
`nanopre` is a zero-dependency, no-`unsafe` implementation of an extremely minimal C-style text preprocessor. At present, `nanopre` is in a minimum viable product state, which is to say that while it is functional and free of bugs to the best of my knowledge, it is not battle-tested, stable, or feature-complete.

## Features
- `Context::define` allows specifying 'macros,' strings of the form `[a-zA-z_][a-zA-Z0-9_]*` which should be replaced by an arbitrary string everywhere they appear in the input (when surrounded by word boundaries). Macros cannot currently accept arguments or expand to other macros or preprocessor directives.
-  The directives `#if`, `#elseif`, `#else`, and `#endif` allow the conditional inclusion of code based on the evaluation of simple boolean expressions. The literals are `0` and `1`, the supported operators are `&&`, `||`, and `!`, and parentheses may be used for grouping. Evaluation is left-associative. While no other tokens are permitted in these expressions, they are evaluated after macro substitution, so you may use macros which evaluate to `1` or `0` as a substitute for variables.

## Planned Features
- `#define` for defining macros within the input.
- Support for `#include`, with a user-specified map from paths to content.
- A way to optionally trap unknown preprocessor directives. Currently, these are left as is.
- More descriptive error handling, depending on how complex this is to implement.

## Credits
`nanopre` is in part inspired by Diggsey's [`minipre`](`https://github.com/Diggsey/minipre`), which I used before building this project.