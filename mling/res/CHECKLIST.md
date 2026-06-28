> [!NOTE]
>
> You are using `mling` to create your `mingling` project. Before proceeding, please fill in your project information.

## Question 1: Fill in your project information

Use `snake-case` style to fill in the project name. It will serve as your `crate` name:

```name
my-cli
```

Describe your `program` in one sentence:

```description
This is a command-line program
```

## Question 2: Which Mingling version do you need?

> [!TIP]
>
> You must select **EXACTLY ONE** option

- [x] `ver:0.2` Based on the `Mingling@0.2.x` template

## Question 3: What is your project layout style?

> [!TIP]
>
> You must select **EXACTLY ONE** option

- [x] `base:flat` Flat mode (Simple)

```
# The most basic mode
./main.rs
./foo.rs
./bar.rs
./globals.rs
```

- [ ] `base:modularity` Modular mode

```
├─ module1/           # Stores private concepts internally
│  ├── mod.rs         # Command registration
│  ├── chains/        # Core logic
│  └── renderers/     # Result rendering
├─ module2/
│  ├── mod.rs         # Command registration
│  ├── chains/        # Core logic
│  └── renderers/     # Result rendering
├─ global/            # Stores public concepts in the global directory
│  ├── errors/        # Global Errors
│  ├── setups/        # Setups
│  └── resources/     # Global Resources
└─ main.rs
```

- [ ] `base:command` Command mode

```
#  Stores all commands
├─ commands/               # Organizes subcommands by hierarchy
│  ├── remote/add.rs       # "remote add"
│  ├── remote/rm.rs        # "remote rm"
│  ├── remote.rs           # Not a command,
│  │                           but a local implementation for
│  │                           the remote series of commands
│  ├── foo.rs              # "foo"
│  └── bar.rs              # "bar"
├─ errors/                 # Global Errors
├─ resources/              # Global Resources
├─ setups/                 # Setups
└─ main.rs
```

## Question 4: What additional features do you need to enable?

> [!TIP]
>
> All of the following options are **OPTIONAL**

**Pre:**

- [x] `exec:git_init` Initialize this project as a git repository (requires `git` to be installed)

**Features:**

- [x] `feat:extra_macros` Extra macro support
- [x] `feat:dispatch_tree` Compile-time dispatch tree (solidify all commands)
- [ ] `feat:structural_renderer` Structural renderer (JSON/YAML) for feature output
- [ ] `feat:comp` Dynamic completion system
- [ ] `feat:clap` Clap support
- [ ] `feat:parser` Mingling Picker parser support
- [ ] `feat:async` Async support

**Implementations:**

- [ ] `impl:exit_code` Introduce exit code control
- [ ] `impl:fallback` Fallback functionality when no command is given

---

After filling in the above, execute the following command to create the project:

```
~# mling gen
```
