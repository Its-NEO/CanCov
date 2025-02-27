# Introduction
A canonical cover for F (a set of functional dependencies on a relation scheme) is a set of dependencies such that F logically implies all dependencies in F.

This program finds the canonical cover for a given set of functional dependencies.

# Installation
## Requirements
[rust](https://www.rust-lang.org/tools/install) and its own set of dependencies 

## Setup
```bash
git clone https://github.com/Its-NEO/CanCov
cd CanCov
cargo run
```

# Usage
Enter the functional dependencies as such:
```bash
x->w
w,z->x,y
y->w,x,z
```

To end entering further inputs, just press enter with an empty line.

Output:
```
x->w
w,z->y
y->x,z
```
