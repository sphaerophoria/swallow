# Swallow

Swallow is a tool that generates a compilation database for clang tooling.

It simulates a bulid by "swallowing" build commands. If it sees a GCC command
that it thinks generates a compilation unit, it prevents it from running and 
logs it into a compile\_commands.json. 

Sometimes clang based tooling isn't smart enough to do the right thing. If the
`-c` command is used, it will attempt to clangify the command to help other 
tooling out. See `swallow --help` for more info after building.

# Building

Needs rust, camke and a c compiler.

```
mkdir build
cd build
cmake ..
make
make install
```

# Credits
This project draws heavily on code from https://github.com/rizsotto/Bear, thanks
for the hard work!
