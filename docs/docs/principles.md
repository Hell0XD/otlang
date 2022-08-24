---
title: Principles
sidebar_position: 7
---

## Linked lists

Because there are no vectors in **OT** you have to use linked lists to represent data.
Pairs are great way to do so. Just chain the pairs and you have linked list.

```
    (pair 1 (pair 2 (pair 3 !)))
```

## Pattern matching

Pattern matching is really powerfull tool, so use it to your's advantage.

```
    (defn foo
        [0] "Is zero"
        [1] "Is One"
        [n] "I dont know what it is"
    )
```