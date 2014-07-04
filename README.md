Game of Life [![Build Status](https://api.travis-ci.org/Arcterus/game-of-life.svg?branch=master)](https://travis-ci.org/Arcterus/game-of-life)
================

An implementation of Conway's Game of Life in
[Rust](https://github.com/rust-lang/rust) using
[Piston](https://github.com/PistonDevelopers/piston).

Build Instructions
------------------

```
make
```

Game Instructions
-----------------

![screenshot](https://raw.githubusercontent.com/arcterus/game-of-life/master/game-of-life.png)

When the game starts up, you click on areas with your mouse where you'd like
cells to appear.  When you've finished making cells, hit either the ```return```
key or the ```p``` key to watch the game run.  Both of these keys may be used
to pause the game.  If you'd like to restart the game, press ```r```.

Contribute
----------

I'd appreciate any contributions, especially for fixing bugs and improving the
UI.  Contributions target Rust's master branch until Rust 1.0 is released.

Credits
-------

* Arcterus (this entire project)
* MagentaCompanion (the original thndr of making the Game of Life in Java)

License
-------

Copyright (C) 2014 by Arcterus.  
This project is licensed under the MPL v2.0.  See ```LICENSE``` for more
details.
