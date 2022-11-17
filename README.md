# rot360

## automatic display rotation using built-in accelerometer

Automatic rotate modern Linux desktop screen and input devices. Handy for
convertible touchscreen notebooks like HP Spectre x360, Lenovo IdeaPad Flex or Linux phone like Pinephone.

Compatible with [sway](http://swaywm.org/).

Rust language and the cargo package manager are required to build the binary.

```
$ git clone https://github.com/7Zifle/rot360
$ cd rot360 && cargo build --release
$ cp target/release/rot360  /usr/bin/rot360
```

For Sway map your input to the output device:

```

$ swaymsg input <INPUTDEVICE> map_to_output <OUTPUTDEVICE>

```

Call rot360 from sway configuration file ~/.config/sway/config:

```

exec rot360

```

there are the following args.

```
-o, --oneshot
-s, --sleep <SLEEP>                                [default: 1000]
-d, --display <DISPLAY>                            [default: eDP-1]
    --touchscreen <TOUCHSCREEN>
-t, --threshold <THRESHOLD>                        [default: 0.2]
    --normalization-factor <NORMALIZATION_FACTOR>  [default: 1000000]
    --keyboard
-h, --help                                         Print help information
-V, --version                                      Print version information

```
