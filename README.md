# Numerical-Splines
A tiny bezier-spline editor made for my Numerical Calculus class

## Group Members
|NOME|NUSP|
|----|----|
|Pedro Henrique Borges Monici|10816732|
|Gabriel Victor Cardoso Fernandes|11878296|
|Guilherme Machado Rios|11222849|
|Daniel Carvalho Dantas|10685702|

## About
This editor implements Bezier Splines with the option to toggle between
the De Casltejau's and the Bernstein Polynomial Form algorithms. Additionally,
you'll find the bounding box and tight bounding box implemented for each
curve segment.

To run, you'll need to have the rust toolchain installed. Just type
```bash
cargo run --release
```

and you're good to go :)

## Controls
|key|action|
|----|----|
|left mouse button| add point or drag existing point|
|right mouse button| delete hovered point|
|g| toggle background grid |
|b| toggle bounding boxes (blue is regular gold is tight) |
|m| toggle algorithm |

## Credits
Curves, splines and NURBs are a fascinating topic and i wish i had more time
to play with them. Below are some materials i studied to implement this project

- [The Beauty of Bezier Curves](https://www.youtube.com/watch?v=aVwxzDHniEw)
- [Pomax's primer on Bézier Curves](https://pomax.github.io/bezierinfo/)
- The NURBs book

Made with [macroquad](https://github.com/not-fl3/macroquad)