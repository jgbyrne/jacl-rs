# JACL: Jack's Application Config Language

`JACL` (pronounced *Jackal*) is an experimental new configuration language.

It is intended to be more expressive than `TOML`, more minimal than `YAML`, and more readable than `JSON`.

### Overview

Here is a simple `JACL` file. It describes a config for an IRC Client.

```
servers {
    freenode {
        name = "Freenode"
	addr = "chat.freenode.org"
	port = 6667
	nick = "martin"
    }

    rizon {
        name = "Rizon"
	addr = "irc.rizon.net"
	port = 9999
	nick = "sarah"
    }

    default = freenode
}

filters [
    {
        server = @freenode
	user   = "matthew"
	action = "ignore"
    }

    {
        server = @rizon
	user   = "carl"
	action = "highlight"
    }
]
```

A pair of curly braces creates an `Object`, the core data structure in `JACL`.  
Objects contain two types of data - other data structures, and data values.  
These two types of data are stored in two separate ordered hash maps inside each Object.

The structures stored within an object are called its `entries`.  
The values stored within an object are called its `properties`.

In the example above, `freenode` and `rizon` refer to entries within the Object `servers`, while `default` refers to a property.

### Values

Property values in `JACL` may be any one of the following datatypes.

The literal datatypes in `JACL` are as follows:

* `String` - String values like `"martin"`
* `Integer` - Integer values like `9999`
* `Float` - Float values like `0.1337`
* `Boolean` - Boolean values - either `true` or `false`

There is just one compound datatype:

* `Tuple` - Sequences of other values like `(4, true, "lapwing")`

There are three reference datatypes:

* `Key` - A reference to an entry within the same structure like `freenode`
* `Var` - A reference to a property within the same structure like `$default`
* `Foreign` - An arbitrary reference like `@rizon`

`Key` and `Var` values are resolved automatically at parse-time and cause errors if they cannot be found in the same structure. The parser makes no effort to resolve `Foreign` values which may be used arbitrarily by the program to attempt to access other data. 

### Structures

There are three types of data structure in `JACL`.

You have already seen the `Object`, which contains both `entries` and `properties`.

The example above also features a `Table`, denoted with square brackets, which may only contain entries.

The third data structure is the `Map`, which is denoted with `{%` and `%}`. It may only contain properties.

There is nothing that either a Table or a Map can do that cannot be done with an Object. However, it can be useful to use the more restricted types to ensure configs are clear and correct.

### Keys and Vars

A `Key` is the name under which an `Entry` is stored. A `Var` is the name under which a `Property` is stored.
Every `Property` has a `Var` but not every `Entry` has a `Key`.

Here are some more complex ways in which entries can be defined.

```
examples {
    atom

    [
        proton
	neutron
	electron
    ]

    country = {%
        name = "United Kingdom"
	continent = @europe
	denonym = "Briton"
    %}
}
```

The `examples` Object contains three entries.

The first, `atom`, is an empty `Entry` - a `Key` with no structure associated.

The second is an anonymous structure. It is a Table which in turn contains three empty entries.

The third entry is a Map. It is also an anonymous structure. However, this structure is bound to a property.

This notation can be considered shorthand for the following:

```
anon {%
    name = "United Kingdom"
    continent = @europe
    denonym = "Briton"
%}

country = anon
```

Be careful, though! The structure is still an anonymous entry within the parent Object even though it has a property binding.

Since Keys and Vars are in different namespaces it is entirely possible to use the same name for a different Entry and Property. This is, however, discouraged for obvious reasons.

### Duplicate and Triplicate

A property can be assigned to an arbritrary number of vars.

```
numbers {
     primes = (2, 3, 5, 7, 11, 13, 17)
     evens, perfects = (6, 28, 496, 8128)
}
```

Another cautionary note: `JACL` does not have tuple destructuring!

```
a, b = (27, 63)
```

Here, both `a` and `b` are bound to the value `(27, 63)`.

Similarly, an entry can be assigned to multiple different keys.

```
mark + jeff + patrick {
    job = "Teacher"
}
```

### Redefining Entries

Multiple definitions can apply to an entry.

```
drivers {
    lewis {
        country = "United Kingdom"
    }

    valtteri {
        country = "Finland"
    }

    max {
        country = "Netherlands"
    }

    lewis + max {
        quick = true
    }
}
```

This works as you would expect. Vars can be overwritten by successive definitions.

When data-structures are redefined they append to previous definitions.

```
cake {
    christmas {
        ingredients [
	    cherries
	]
    }

    easter {
        ingredients [
	    marzipan 
	]
    }

    christmas + easter {
        ingredients [
	    raisins
	]
    }
}
```

This evaluates intuitively to be equivalent to the following:

```
cake {
    christmas {
        ingredients [
	    cherries
	    raisins
	]
    }

    easter {
        ingredients [
	    marzipan 
	    raisins
	]
    }
}
```

### Afterword

Thanks for reading this far! `JACL` is still very much in development but I hope one day it can be useful to many people.

I appreciate suggestions as to how I can make the language better.


