# IcedDobotController
Simple Application to control Dobot in Iced

Icedを用いた簡単なDobotロボットアームの制御GUIアプリである

## What is this?・なにこれ?
An Iced GUI program I made for my graduation research project.
It is used to control a Dobot Magician robot arm.
It is not intended for general use. You can try to edit it for your own purpose,
but I do not guarantee plug-n-play functionality.

本レポジトリーは自分の卒業研究のためのIced GUIプログラムであり, Dobotと言うロボットアームの制御するプログラムである.
Dobot Studioみたいの一般的な使用を意図したものではなく, 自分のユースケースに特化したものである.
Dobot Studioより, 本プログラムはその中のpythonやcode blocksスクリプティングのように動作する.


## Basic usage・使用方法
Needs to be connected to a Dobot Magician for full functionality.
Otherwise enable debug mode in settings, which only will work for experiment mode.
Works on wayland, but might need some dependencies depending on your distribution.
Not tested on Windows at all, most likely needs a different serial communication implementation.

Dobot Magicianと接続する必要があり, それをなければ設定ページで`enable debug mode`をONにしなければならい (`debug mode`にはシーケンサーモードを使用できない).
おそらく追加の依存関係が必要が, Waylandで実行できる. Linux専用である.
Windows環境では全くテストしてないので, 直接にコンパイルと実行することができないことの可能性が高い. 
```
git clone https://github.com/marischou/IcedDobotController.git
cd IcedDobotController
cargo run
```


## Credits・参考文献
1. jerry73204, myself: dobot in rust implementation; https://github.com/marischou/dobot-rust-fx24.git
2. airstrike: dragking dragging library: https://github.com/airstrike/dragking.git
