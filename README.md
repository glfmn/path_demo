![Visual demonstration](img/path_demo.gif)

# Pathfinding in Video Games

An experiment with robotics-based approaches to pathfinding applied to a video game environment.

This demo was created as a graduation requirement from my university and as such the code is not up to my standard for public releases.  The application itself is okay to use, but the code that powers it is messy and is not meant for continous maintenance.  Many things are copy-and-paste.

## Acknowledgements

This work was only possible thanks to the research at the [FSU CISCOR][CISCOR] robotics laboratory, and the help and kindness of many people at FSU and on internet communities such as `r/Rust` and `r/roguelikedev`.  Many thanks goes to all of them.

## Building the Presentation

With the rust tool-chain installed, clone the repository and build it with
cargo:

```
$ git clone https://github.com/glfmn/path_demo.git
$ cd path_demo
$ cargo run --release
```

There are some dependencies which must be installed for [`libtcod`] which can be found on the [`libtcod` README][dependencies].

## Presentation Controls

| Key               | Function                                               |
|:-----------------:|:-------------------------------------------------------|
| `Left Click`      | Place the monster icon (`M`) under the cursor          |
| `Right Click`     | Place the player, or goal, icon (`@`) under the cursor |
| `Enter`           | Step through one iteration of path-finding             |
| `Shift` + `Enter` | Path-find until the final path is found                |
| `Backspace`       | Restart path-finding from the beginning                |
| `Tab`             | Select the next menu option                            |
| `Page UP`         | Select previous menu option                            |
| `Page Down`       | Select the next menu option                            |
| `Space`           | Toggle the current menu option                         |

## Technical Overview

> The code is very messy and rather poorly organized as it was a class project that I had to rush to complete.  I undertook this project to learn more about how games are organized, and to apply some ideas and concepts I had learned in robotics to game A.I..  I believe I succeded in doing so, but understand this code is experimental and has not been vetted through use in an actual game.  With that in mind...

If you are interested in learning more about the methods used, the core algorithm is located behind `lib/path/`.

### Prerequisites

It's best to have a solid understanding of A* before jumping into SBMPO; while SBMPO technically is an approach to designing an algorithm, I have implemented it here with A\*, and will motivate and describe it through A\*.

Learn more at [Red Blob Games](https://www.redblobgames.com/pathfinding/a-star/introduction.html)'s amazing A* introduction.

### Motivation

The approach is called "Sampling-based model-predictive optimization," a technique developed at the [FSU CISCOR][CISCOR] robotics lab.  The technique was invented to overcome challenges in mobile robotics and in controls.

[`libtcod`]: https://github.com/tomassedovic/tcod-rs
[dependencies]: https://github.com/tomassedovic/tcod-rs/blob/master/README.md#how-to-use-this
[releases]: https://github.com/glfmn/path_demo/releases
[CISCOR]: https://www.ciscor.org/

