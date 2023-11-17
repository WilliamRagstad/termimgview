# Terminal Image Viewer

This is a simple image viewer for the terminal. It is written in Rust and uses `crossterm`, `image`, and `clap`.\
Inspiration and alternatives: [tiv](https://github.com/stefanhaustein/TerminalImageViewer), [timg](https://github.com/hzeller/timg), [termimage](https://github.com/nabijaczleweli/termimage), [imgcat](https://github.com/eddieantonio/imgcat), [viu](https://github.com/atanunq/viu), or [pixterm](https://github.com/eliukblau/pixterm).

## Usage
    
```
Usage: termimgview [OPTIONS] <FILE>

Arguments:
  <FILE>  Path to the image file to be displayed

Options:
  -m, --shade-method <SHADE_METHOD>                Shading method [default: blocks]
  -s, --scale <SCALE>                              The scale of the image [default: 1]
  -g, --grayscale                                  Grayscale image?
  -i, --invert                                     Invert image?
  -a, --adjust-aspect-ratio <ADJUST_ASPECT_RATIO>  Adjust aspect ratio [default: 0.47058824]
  -b, --brightness <BRIGHTNESS>                    Brightness of the image [default: 1]
  -r, --hue-rotation <HUE_ROTATION>                Rotate the hue of the image [default: 0]
  -h, --help                                       Print help
  -V, --version                                    Print version

Shade methods:
 - ascii: ' .-:=+*#%@'
 - blocks: ' ░▒▓█'
 - custom: 'your characters here'

Example usage:
 - termimgview .\tests\1.png -s 0.15 -m " -:!|#@@@@@@@@"
 - termimgview .\tests\2.jpg -s 1 -i -m ascii
```

## Installation

### From source
```bash
> git clone https://github.com/WilliamRagstad/termimgview.git
> cd termimgview
> cargo install --path .
```
