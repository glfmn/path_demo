![Visual demonstration](img/path_demo.gif)

# Pathfinding in Video Games

An experiment with robotics-based approaches to pathfinding applied to a video game environment.

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
| `Delete`          | Generate a new map                                     |

[`libtcod`]: https://github.com/tomassedovic/tcod-rs
[dependencies]: https://github.com/tomassedovic/tcod-rs/blob/master/README.md#how-to-use-this
[releases]: https://github.com/glfmn/path_demo/releases
