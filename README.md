![boulder-dash.rs screenshot](https://i.imgur.com/B47GWJm.png)

# Boulder Dash in Rust (using Amethyst game engine)

Boulder Dash is probably my favourite game of all times. I used
to play it a lot on my C-64 when I was a kid. This is how it looked: https://www.youtube.com/watch?v=FiEVfa1OK_o

Since I wanted to try out [amethyst game engine](https://amethyst.rs/), I decided
to try to build a remake/clone of Boulder Dash.

## Status

Some basics work. You can move around and eat diamonds, rocks fall down. My son
actually enjoys it a lot. Maps can be created in a text editor.

Start with `cargo run`.

Use -/+ for zoom level, arrows to move.

**If you're looking for a fun project to hack on, I'd be happy to accept help**. 
I actually don't have a lot of time to work on it myself, so I'd mostly be a
sidekick, I guess.

## Install

### Ubuntu

```
sudo apt-get install libxcb-shape0-dev libxcb-xfixes0-dev mesa-vulkan-drivers
```

Note: If you have an Nvidia GPU instead of `mesa-vulkan-drivers` you will need:

```
sudo add-apt-repository ppa:graphics-drivers/ppa
sudo apt update
sudo apt install libvulkan1
```

## Goals & ideas

* Add animation and sound
* Add all the original behaviors
* Experiment with new behaviors:
  * my son is happy with rocks being harmless
  * push rocks up adds more puzzle-like possibilities
* Add support for BDCFF and other BD-community standards
* Use the power of ECS and try maaaaasive maps.
* Compile down to WASM so it can work in a browser
* Build-in map editor would be nice
* server-side verified highscores?
* multiplayer?
* sky is the limit!


# Alternatives & links

There's a big community of Boulder Dash fans around the world,
and it's a common game to make remakes of. Unfortunately a lot
of remakes suffer bit-rot. Feel free to submit link here:

* http://www.boulder-dash.nl/ - great source of materials
* https://github.com/irmen/bouldercaves - BD clone in Python
