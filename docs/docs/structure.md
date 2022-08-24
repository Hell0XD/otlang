---
sidebar_position: 4
---

# Structure

The compiler compiles all code files in current directory and in any child directory. Minimal structure is to have a `main` file as a entry file.

Only the `main` file can have expressions outside the functions and other files have to contain only functions.

Each file has to have at the beggining module name simularly like in **Golang**. The module names should be same as the file names.

```plaintext title="bar.ot" {1}
    mod bar

    (print "Hello!\n")
```

## Program entry point

The language support top level expressions, therefore the top level is an entry point for OT application.