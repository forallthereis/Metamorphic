![Metamorphic logo](https://github.com/forallthereis/Metamorphic/blob/main/resources/logo.png?raw=true)

![CLI capture](https://github.com/forallthereis/Metamorphic/blob/main/resources/cli_capture.gif?raw=true)

**Metamorphic** is a (largely experimental) tree-decomposition-based solver for integer linear programs (ILPs).
Unlike many other ILP solvers, Metamorphic uses neither branch-and-bound nor any kind of relaxation.
Instead, it uses a dynamic programming algorithm largely inspired by [this paper](https://cdn.aaai.org/ojs/10644/10644-13-14172-1-2-20201228.pdf).
Note that Metamorphic isn't intended to be a general-purpose ILP solver and it can be expected to effectively solve only ILPs with small incidence treewidth.

## What does Metamorphic do?

Metamorphic solves ILPs:
1. That have a low incidence treewidth.
2. In which each variable has explicit, finite lower and upper bounds.

It does this in the following order:
1. It reads an ILP instance from the given input file, builds an incidence graph for it. Supported input formats are LP and MPS.
2. It computes a nice tree decomposition for the incidence graph.
3. It solves the instance using dynamic programming over the computed nice tree decomposition.
4. If the instance is solved to optimality/feasibility, then an optimal/feasible solution is saved as a Gurobi SOL file.

## How to use Metamorphic?

As of now, we don't ship any precompiled binaries, so you'll have to download the sources and compile them yourself...
The good news is that running `cargo build --release` is all you'll need to do.
If you're not familiar with Cargo, you can find out more [here](https://doc.rust-lang.org/cargo/).

Once you have compiled Metamorphic (which may take some time), you have two options: you can either run it in _command-line mode_ or in _interactive mode_.

#### Command-line mode

Command-line mode is how most command-line programs are used: you call the program by typing its name and listing all its arguments one after another in a command line.
For example:

```
metamorphic my_ilp_instance.mps tcauto logfile logmd
```

This means: solve the ILP instance from `my_ilp_instance.mps` using an automatically determined number of worker threads while saving logs in the Markdown format on a disk.
Call `metamorphic --help` for more detail.

#### Interactive mode

To launch the interactive mode, call `metamorphic` without any additional arguments.
The demo of the interactive mode is shown in the animation above.

## Important note

Metamorphic is now in pre-release.
It works as intended and solves problems but breaking changes in its interface are possible in the coming future.
