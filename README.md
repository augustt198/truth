# truth

A boolean expression parser and evaluator.

### Examples:

Expression: **`!A`**

```
> Truth table:
A    Result

0    1
1    0
> Parsed tree:
Operation { components: [Component { value: Var(A), negated: true }], ops: [] }
> Variables: [A]
```

Expression: **`X & Y`**

```
> Truth table:
X    Y    Result

0    0    0
0    1    0
1    0    0
1    1    1
> Parsed tree:
Operation { components: [Component { value: Var(X), negated: false }, Component { value: Var(Y), negated: false }], ops: [Token { token_type: And, col: 3, line: 1 }] }
> Variables: [X, Y]
```

Expression: **`(A & B) | (C ^ D)`**

```
> Truth table:
A    B    C    D    Result

0    0    0    0    0
0    0    0    1    1
0    0    1    0    1
0    0    1    1    0
0    1    0    0    0
0    1    0    1    1
0    1    1    0    1
0    1    1    1    0
1    0    0    0    0
1    0    0    1    1
1    0    1    0    1
1    0    1    1    0
1    1    0    0    1
1    1    0    1    1
1    1    1    0    1
1    1    1    1    1
> Parsed tree:
Operation { components: [Component { value: Expr(Operation { components: [Component { value: Var(A), negated: false }, Component { value: Var(B), negated: false }], ops: [Token { token_type: And, col: 4, line: 1 }] }), negated: false }, Component { value: Expr(Operation { components: [Component { value: Var(C), negated: false }, Component { value: Var(D), negated: false }], ops: [Token { token_type: Xor, col: 14, line: 1 }] }), negated: false }], ops: [Token { token_type: Or, col: 9, line: 1 }] }
> Variables: [A, B, C, D]
```

---

**Note:** the AND (`&`), OR (`|`) and XOR (`^`) operators have the same precedence.
