---
sidebar_position: 5
---

# Common Programming Concepts


## Comments

Just like in python, OT uses `#` for single line comments.

```
    # I'am a comment!
```


## Variables

All variables are immutable in **OT**. There is no need for mutable variables because controlls like `if` are expressions and cycles does not exist.

To declare a variable use a keyword `def` followed by variable name and an expression.

```
    (def bar 5) # immediate assignment
    (def foo (+ 4 6)) # expression assignment
```

Variables are scoped and you cannot define 2 variables with same name.

## Conditions

The main way for to write conditional code is `if`. As said above `if` is an expression therefore it always has to contain code for truthy and falsy condition.

To create an `if` expression use keyword `if` followed by condition, truthy expression and falsy expression.

```
    (if (> 5 4) 
        "5 is more than 4" 
        "5 is less than 4"
    )
```

## Functions

Function have to have compile-time known number of arguments and arguments can be pattern matched.

This allows us to declare functions like in mathematics. For example take a look at fibonacci number defined on Wikipedia.

Functions can't access any variables outside their scope.

```
    F₀ = 0
    F₁ = 1
    Fₙ = Fₙ₋₁ + Fₙ₋₂
```

We can define it in **OT** like this:

```
    (defn F
        [0] 0
        [1] 1
        [n] (+ (F (- n 1)) (F (- n 2)))
    )
```

The function can match any number of arguments and can match the values aswell. You can imagine the function definition as switch statement.

After every list of arguments there must be an expression that is evaluated when the arguments match.

## Blocks

Sometimes you need more than one line to express your code. For that there is `do` keyword, all arguments of `do` are evaluated sequentially and the last expression is returned, because `do` is expression as well.

This kind of blocks create another scope for variables.

```
    (defn println [x] (do
        (print x)
        (print "\n")
    ))
```

With this we can create multiline functions.

# IO

To print something into stdout there is buildin function `print`. It takes 1 argument and prints its value.

```
    (print "Hi!")
```


