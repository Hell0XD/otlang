---
sidebar_position: 6
title: "Types"
---


## Integer
Integer type is repesented as 32bit integer.

```
    5
    -6
    0xFF
    0b0101101
```

## String
UTF-8 encoded string.

```
    "Foo"
    "\t bar \n"
```
## Atom
Atom is constant which value is their own name.

```
    :ok
    :err
    :true
    :false
```

## Null
A null, usefull for ending an linked list

```
    !
```

## Pair
Pair is a construct that contains 2 values. They are mostly used to create a linked list.

```
    (pair 4 5)
    (pair 1 (pair 2 (pair 3 !)))
```

## Lambda
Lambdas are like functions that are dynamically created and can capture context. Unlike functions they can not pattern match their arguments.

```
    (lambda [x] (+ x 1))

    (def y 10)
    (lambda [z] (+ y z))
```


