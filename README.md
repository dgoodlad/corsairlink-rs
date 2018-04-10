# Corsair Link

Work-in-progress Rust library to interact with Corsair Link devices.

Given this is a pet project of mine mainly to learn Rust, and to have fun tinkering with low-level protocol reverse engineering, you probably shouldn't use it for anything serious.

## Device Support

* *h110i* firmware v2.0.00 (device id 0x42)

Future work will likely go towards supporting the HX750i in my PC.

## Credits

Let's be honest, I'm re-inventing the wheel here. While I've done my fair share of USB traces to gather my own data about the devices I own, I wouldn't have gotten very far without the hard work of others:

* http://forum.corsair.com/forums/showthread.php?t=120092
* https://github.com/audiohacked/OpenCorsairLink

## License

MIT.

See LICENSE.txt
