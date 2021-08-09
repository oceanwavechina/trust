
# TCP 的 keep-alive 机制

关于 TCP 的 keep-alive 机制，在 rfc1122 中有相应的要求。因为人们对于把 keep-alive 是否应该放到 TCP 里边有争议，所以这个机制不是 **MUST** 包含的。如果实现的话，默认也是要求关闭。 

<br>

## 1. keep-alive 的实现要求
----
<br>

keep-alive 实现的机制如下：

1. keep-alive 包的发送，只能在连接有 **一段时间** 没有 **收到** 包。

2. **一段时间** 要求最少不低于 2 个小时。

3. keep-alive 的探测包不能包含有效数据。因为 keep-alive 的 Sequence number 是比 SND.NXT 少 1，
   
   即 ```SEG.SEQ = SND.NXT-1```，这个要在对方接收窗口**左侧**， 因为没有数据，不能占用正常的窗口

4. keep-alive 机制建议只在 server 端开启。为了释放连接已经断开的 client 资源。

需要注意的是，如果 keep-alive 失败，并不能断定这个连接一定不可用。


<br>

## 2. keep-alive 在不同场景下的表现
----
<br>

keep-alive 在进行探测时可能会发生 4 种情况： 正常， 对端崩溃， 对端重启， 中间网络异常。

在对比 对端重启 和 中间网络异常 时， 其实更能深入理解： tcp 是端对端的状态协议。什么意思？

1. tcp 连接的两端只是分别保存了对方的状态信息。

2. 但是它并不知道数据在中间传递时转了几个弯

所以如果没有 keep-alive 机制，当中间网异常时，tcp 连接是无法知道的。

而 keep-alive 机制其实只是在检测两中情况：

1. 对端是否还保存这我们的连接状态信息 （关机和重启后所保存的状态就都没有了）

2. 中间的路是否还能够走通。


我们的例子在 test/server.py 和 test/client.py 里边。其中 server 端负责发送 keep-alive prob。

<br>

### 2.1. 正常情况

<br>

这种模拟的场景是 在本地起 client 和 server。

server 监听的端口是 **2000**， 并且 keep-alive 是在 server 端开启的。

ip 用的是 127.0.0.1

<br>

``` s
liuyanan@localhost:~$ sudo tshark -i lo tcp port 2000
Running as user "root" and group "root". This could be dangerous.
Capturing on 'Loopback'
    1 0.000000000    127.0.0.1 → 127.0.0.1    TCP 66 36536 → 2000 [SYN] Seq=0 Win=65495 Len=0 MSS=65495 SACK_PERM=1 WS=128
    2 0.000011937    127.0.0.1 → 127.0.0.1    TCP 66 2000 → 36536 [SYN, ACK] Seq=0 Ack=1 Win=65495 Len=0 MSS=65495 SACK_PERM=1 WS=128
    3 0.000021844    127.0.0.1 → 127.0.0.1    TCP 54 36536 → 2000 [ACK] Seq=1 Ack=1 Win=65536 Len=0
    
    4 0.000332898    127.0.0.1 → 127.0.0.1    TCP 66 36536 → 2000 [PSH, ACK] Seq=1 Ack=1 Win=65536 Len=12
    5 0.000339670    127.0.0.1 → 127.0.0.1    TCP 54 2000 → 36536 [ACK] Seq=1 Ack=13 Win=65536 Len=0
    6 0.000357609    127.0.0.1 → 127.0.0.1    TCP 66 2000 → 36536 [PSH, ACK] Seq=1 Ack=13 Win=65536 Len=12
    7 0.000416758    127.0.0.1 → 127.0.0.1    TCP 54 36536 → 2000 [ACK] Seq=13 Ack=13 Win=65536 Len=0
    
    8 5.120253274    127.0.0.1 → 127.0.0.1    TCP 54 [TCP Keep-Alive] 2000 → 36536 [ACK] Seq=12 Ack=13 Win=65536 Len=0
    9 5.120285491    127.0.0.1 → 127.0.0.1    TCP 54 [TCP Keep-Alive ACK] 36536 → 2000 [ACK] Seq=13 Ack=13 Win=65536 Len=0
   10 10.240306851    127.0.0.1 → 127.0.0.1    TCP 54 [TCP Keep-Alive] 2000 → 36536 [ACK] Seq=12 Ack=13 Win=65536 Len=0
   11 10.240327631    127.0.0.1 → 127.0.0.1    TCP 54 [TCP Keep-Alive ACK] 36536 → 2000 [ACK] Seq=13 Ack=13 Win=65536 Len=0
   
   12 12.318998074    127.0.0.1 → 127.0.0.1    TCP 54 36536 → 2000 [FIN, ACK] Seq=13 Ack=13 Win=65536 Len=0
   13 12.319047416    127.0.0.1 → 127.0.0.1    TCP 54 2000 → 36536 [FIN, ACK] Seq=13 Ack=14 Win=65536 Len=0
   14 12.319054011    127.0.0.1 → 127.0.0.1    TCP 54 36536 → 2000 [ACK] Seq=14 Ack=14 Win=65536 Len=0
```

<br>

上边可以分成 4 部分，分别是 三次握手建立连接， client发送数据 且 server echo返回， keep-alive ， 连接关闭。

可以看第8行的 sequence number 比 client 端 seq=13 要小 1。并且两次 keep-alive 都是如此。

keep-alive 的间隔如程序中是每 5s 一次



<br>

### 2.2. 一端崩溃(断电)

<br>

这一次我们在两个虚拟机中分别启动 server 和 client， 然后client 机器断电。

<br>

``` s
liuyanan@localhost:~$ sudo tshark -i ens33 tcp port 2000
Running as user "root" and group "root". This could be dangerous.
Capturing on 'ens33'
    1 0.000000000 172.17.79.33 → 172.17.79.12 TCP 74 35410 → 2000 [SYN] Seq=0 Win=64240 Len=0 MSS=1460 SACK_PERM=1 TSval=769995180 TSecr=0 WS=128
    2 0.000069267 172.17.79.12 → 172.17.79.33 TCP 66 2000 → 35410 [SYN, ACK] Seq=0 Ack=1 Win=64240 Len=0 MSS=1460 SACK_PERM=1 WS=128
    3 0.000737526 172.17.79.33 → 172.17.79.12 TCP 60 35410 → 2000 [ACK] Seq=1 Ack=1 Win=64256 Len=0
    4 0.001757747 172.17.79.33 → 172.17.79.12 TCP 66 35410 → 2000 [PSH, ACK] Seq=1 Ack=1 Win=64256 Len=12
    5 0.001773438 172.17.79.12 → 172.17.79.33 TCP 54 2000 → 35410 [ACK] Seq=1 Ack=13 Win=64256 Len=0
    6 0.001917489 172.17.79.12 → 172.17.79.33 TCP 66 2000 → 35410 [PSH, ACK] Seq=1 Ack=13 Win=64256 Len=12
    7 0.003187134 172.17.79.33 → 172.17.79.12 TCP 60 35410 → 2000 [ACK] Seq=13 Ack=13 Win=64256 Len=0
    
    8 5.555857854 172.17.79.12 → 172.17.79.33 TCP 54 [TCP Keep-Alive] 2000 → 35410 [ACK] Seq=12 Ack=13 Win=64256 Len=0
    9 5.556211591 172.17.79.33 → 172.17.79.12 TCP 60 [TCP Keep-Alive ACK] 35410 → 2000 [ACK] Seq=13 Ack=13 Win=64256 Len=0
   10 10.676030749 172.17.79.12 → 172.17.79.33 TCP 54 [TCP Keep-Alive] 2000 → 35410 [ACK] Seq=12 Ack=13 Win=64256 Len=0
   11 10.676408850 172.17.79.33 → 172.17.79.12 TCP 60 [TCP Keep-Alive ACK] 35410 → 2000 [ACK] Seq=13 Ack=13 Win=64256 Len=0
   12 15.795865061 172.17.79.12 → 172.17.79.33 TCP 54 [TCP Keep-Alive] 2000 → 35410 [ACK] Seq=12 Ack=13 Win=64256 Len=0
   13 15.796249643 172.17.79.33 → 172.17.79.12 TCP 60 [TCP Keep-Alive ACK] 35410 → 2000 [ACK] Seq=13 Ack=13 Win=64256 Len=0
   14 20.916086398 172.17.79.12 → 172.17.79.33 TCP 54 [TCP Keep-Alive] 2000 → 35410 [ACK] Seq=12 Ack=13 Win=64256 Len=0
   
   15 22.932092511 172.17.79.12 → 172.17.79.33 TCP 54 [TCP Keep-Alive] 2000 → 35410 [ACK] Seq=12 Ack=13 Win=64256 Len=0
   16 24.947832905 172.17.79.12 → 172.17.79.33 TCP 54 [TCP Keep-Alive] 2000 → 35410 [ACK] Seq=12 Ack=13 Win=64256 Len=0
   17 26.964271670 172.17.79.12 → 172.17.79.33 TCP 54 [TCP Keep-Alive] 2000 → 35410 [ACK] Seq=12 Ack=13 Win=64256 Len=0
   18 28.979906467 172.17.79.12 → 172.17.79.33 TCP 54 [TCP Keep-Alive] 2000 → 35410 [ACK] Seq=12 Ack=13 Win=64256 Len=0
   19 30.996106847 172.17.79.12 → 172.17.79.33 TCP 54 [TCP Keep-Alive] 2000 → 35410 [ACK] Seq=12 Ack=13 Win=64256 Len=0
   20 33.012033649 172.17.79.12 → 172.17.79.33 TCP 54 [TCP Keep-Alive] 2000 → 35410 [ACK] Seq=12 Ack=13 Win=64256 Len=0
   21 35.027643579 172.17.79.12 → 172.17.79.33 TCP 54 [TCP Keep-Alive] 2000 → 35410 [ACK] Seq=12 Ack=13 Win=64256 Len=0
   22 37.573673887 172.17.79.12 → 172.17.79.33 TCP 54 [TCP Keep-Alive] 2000 → 35410 [ACK] Seq=12 Ack=13 Win=64256 Len=0
   
   23 39.589727273 172.17.79.12 → 172.17.79.33 TCP 54 2000 → 35410 [RST, ACK] Seq=13 Ack=13 Win=64256 Len=0
```

<br>

在 最后一个 keep-alive 没有得到 ACK， 于是在每隔 2s 发了 9 次之后，server端 发送 RST 主动关闭了连接

tcp_keepalive_probes 配置中默认是 9 次

<br>

### 2.3. 一端崩溃(断电重启)

<br>

这一次我们在两个虚拟机中分别启动 server 和 client， 然后client 机器断电， 然后在重启

为了等待重启，我们把检测时间设置的长了些

<br>

```
liuyanan@localhost:~$ sudo tshark -i ens33 tcp port 2000
Running as user "root" and group "root". This could be dangerous.
Capturing on 'ens33'
    1 0.000000000 172.17.79.33 → 172.17.79.12 TCP 74 43756 → 2000 [SYN] Seq=0 Win=64240 Len=0 MSS=1460 SACK_PERM=1 TSval=1515322189 TSecr=0 WS=128
    2 0.000024915 172.17.79.12 → 172.17.79.33 TCP 66 2000 → 43756 [SYN, ACK] Seq=0 Ack=1 Win=64240 Len=0 MSS=1460 SACK_PERM=1 WS=128
    3 0.001134081 172.17.79.33 → 172.17.79.12 TCP 60 43756 → 2000 [ACK] Seq=1 Ack=1 Win=64256 Len=0
    4 0.001153590 172.17.79.33 → 172.17.79.12 TCP 66 43756 → 2000 [PSH, ACK] Seq=1 Ack=1 Win=64256 Len=12
    5 0.001160501 172.17.79.12 → 172.17.79.33 TCP 54 2000 → 43756 [ACK] Seq=1 Ack=13 Win=64256 Len=0
    6 0.001374091 172.17.79.12 → 172.17.79.33 TCP 66 2000 → 43756 [PSH, ACK] Seq=1 Ack=13 Win=64256 Len=12
    7 0.003285321 172.17.79.33 → 172.17.79.12 TCP 60 43756 → 2000 [ACK] Seq=13 Ack=13 Win=64256 Len=0
    8 61.081239027 172.17.79.12 → 172.17.79.33 TCP 54 [TCP Keep-Alive] 2000 → 43756 [ACK] Seq=12 Ack=13 Win=64256 Len=0
    9 122.521306650 172.17.79.12 → 172.17.79.33 TCP 54 [TCP Keep-Alive] 2000 → 43756 [ACK] Seq=12 Ack=13 Win=64256 Len=0
   10 122.522899938 172.17.79.33 → 172.17.79.12 TCP 60 43756 → 2000 [RST] Seq=13 Win=0 Len=0
```

<br>

第 10 行，当 client 端起来后，收到了 server 端的 keep-alive 通知，因为此时 client 没有保存 server 端对应的连接信息，于是返回 RST 重置了该连接。

<br>

### 2.4. 一端崩溃(kill -9)

<br>

这种情况下操作系统会回收该进程的资源，所以会看到正常的连接关闭流程。

<br>

``` s
liuyanan@localhost:~$ sudo tshark -i lo tcp port 2000
Running as user "root" and group "root". This could be dangerous.
Capturing on 'Loopback'
    1 0.000000000 172.17.79.12 → 172.17.79.12 TCP 66 34646 → 2000 [SYN] Seq=0 Win=65495 Len=0 MSS=65495 SACK_PERM=1 WS=128
    2 0.000027833 172.17.79.12 → 172.17.79.12 TCP 66 2000 → 34646 [SYN, ACK] Seq=0 Ack=1 Win=65495 Len=0 MSS=65495 SACK_PERM=1 WS=128
    3 0.000041006 172.17.79.12 → 172.17.79.12 TCP 54 34646 → 2000 [ACK] Seq=1 Ack=1 Win=65536 Len=0
    4 0.000428052 172.17.79.12 → 172.17.79.12 TCP 66 34646 → 2000 [PSH, ACK] Seq=1 Ack=1 Win=65536 Len=12
    5 0.000438094 172.17.79.12 → 172.17.79.12 TCP 54 2000 → 34646 [ACK] Seq=1 Ack=13 Win=65536 Len=0
    6 0.000463065 172.17.79.12 → 172.17.79.12 TCP 66 2000 → 34646 [PSH, ACK] Seq=1 Ack=13 Win=65536 Len=12
    7 0.000475009 172.17.79.12 → 172.17.79.12 TCP 54 34646 → 2000 [ACK] Seq=13 Ack=13 Win=65536 Len=0
    8 5.020553738 172.17.79.12 → 172.17.79.12 TCP 54 [TCP Keep-Alive] 2000 → 34646 [ACK] Seq=12 Ack=13 Win=65536 Len=0
    9 5.020597744 172.17.79.12 → 172.17.79.12 TCP 54 [TCP Keep-Alive ACK] 34646 → 2000 [ACK] Seq=13 Ack=13 Win=65536 Len=0
   10 7.998521592 172.17.79.12 → 172.17.79.12 TCP 54 34646 → 2000 [FIN, ACK] Seq=13 Ack=13 Win=65536 Len=0
   11 7.998574105 172.17.79.12 → 172.17.79.12 TCP 54 2000 → 34646 [FIN, ACK] Seq=13 Ack=14 Win=65536 Len=0
   12 7.998580875 172.17.79.12 → 172.17.79.12 TCP 54 34646 → 2000 [ACK] Seq=14 Ack=14 Win=65536 Len=0
```

<br>

上边可以分成 4 部分，分别是 三次握手建立连接， client发送数据 且 server echo返回， keep-alive ， 连接关闭。

可以看第8行的 sequence number 比 client 端 seq=13 要小 1。并且两次 keep-alive 都是如此。

keep-alive 的间隔如程序中是每 5s 一次

<br>

### 2.5. 中间网络断开(没有开keep-alive)

<br>

这种情况，我们使用 vpn，然后用手机流量 并且 开热点，等 cs 连接 好后把热点关闭

在两台机器上，使用 ```nc -l 1234``` 和  ```nc 172.17.60.78 1234``` 分别做 server 和 client

然后用 tshark 抓取该vpn网卡的包 ```sudo tshark -i utun4 -Y "tcp.port==1234"```

<br>

``` s
 sudo tshark -i utun4 -Y "tcp.port==1234"
Capturing on 'utun4'
   41  27.494570 172.17.93.68 → 172.17.60.78 TCP 68 59145 → 1234 [SYN, ECN, CWR] Seq=0 Win=65535 Len=0 MSS=1360 WS=64 TSval=3278860858 TSecr=0 SACK_PERM=1
   42  27.567461 172.17.60.78 → 172.17.93.68 TCP 64 1234 → 59145 [SYN, ACK, ECN] Seq=0 Ack=1 Win=14480 Len=0 MSS=1460 SACK_PERM=1 TSval=974266049 TSecr=3278860858 WS=128
   43  27.567531 172.17.93.68 → 172.17.60.78 TCP 56 59145 → 1234 [ACK] Seq=1 Ack=1 Win=132096 Len=0 TSval=3278860930 TSecr=974266049
   44  30.774443 172.17.93.68 → 172.17.60.78 TCP 58 59145 → 1234 [PSH, ACK] Seq=1 Ack=1 Win=132096 Len=2 TSval=3278864119 TSecr=974266049
   45  30.841749 172.17.60.78 → 172.17.93.68 TCP 56 1234 → 59145 [ACK] Seq=1 Ack=3 Win=14592 Len=0 TSval=974269329 TSecr=3278864119
   48  44.000424 172.17.93.68 → 172.17.60.78 TCP 58 59145 → 1234 [PSH, ACK] Seq=3 Ack=1 Win=132096 Len=2 TSval=3278877256 TSecr=974269329
   49  44.245408 172.17.93.68 → 172.17.60.78 TCP 58 [TCP Retransmission] 59145 → 1234 [PSH, ACK] Seq=3 Ack=1 Win=132096 Len=2 TSval=3278877498 TSecr=974269329
   50  44.634235 172.17.93.68 → 172.17.60.78 TCP 58 [TCP Retransmission] 59145 → 1234 [PSH, ACK] Seq=3 Ack=1 Win=132096 Len=2 TSval=3278877883 TSecr=974269329
   51  45.206241 172.17.93.68 → 172.17.60.78 TCP 58 [TCP Retransmission] 59145 → 1234 [PSH, ACK] Seq=3 Ack=1 Win=132096 Len=2 TSval=3278878453 TSecr=974269329
   52  46.155233 172.17.93.68 → 172.17.60.78 TCP 58 [TCP Retransmission] 59145 → 1234 [PSH, ACK] Seq=3 Ack=1 Win=132096 Len=2 TSval=3278879393 TSecr=974269329
   53  47.845373 172.17.93.68 → 172.17.60.78 TCP 58 [TCP Retransmission] 59145 → 1234 [PSH, ACK] Seq=3 Ack=1 Win=132096 Len=2 TSval=3278881074 TSecr=974269329
   54  51.033152 172.17.93.68 → 172.17.60.78 TCP 58 [TCP Retransmission] 59145 → 1234 [PSH, ACK] Seq=3 Ack=1 Win=132096 Len=2 TSval=3278884234 TSecr=974269329
   55  55.453328 172.17.93.68 → 172.17.60.78 TCP 58 [TCP Retransmission] 59145 → 1234 [PSH, ACK] Seq=3 Ack=1 Win=132096 Len=2 TSval=3278888626 TSecr=974269329
   56  64.108963 172.17.93.68 → 172.17.60.78 TCP 58 [TCP Retransmission] 59145 → 1234 [PSH, ACK] Seq=3 Ack=1 Win=132096 Len=2 TSval=3278897210 TSecr=974269329
   57  72.759236 172.17.93.68 → 172.17.60.78 TCP 58 [TCP Retransmission] 59145 → 1234 [PSH, ACK] Seq=3 Ack=1 Win=132096 Len=2 TSval=3278905794 TSecr=974269329
   58  81.415732 172.17.93.68 → 172.17.60.78 TCP 58 [TCP Retransmission] 59145 → 1234 [PSH, ACK] Seq=3 Ack=1 Win=132096 Len=2 TSval=3278914378 TSecr=974269329
   59  90.067713 172.17.93.68 → 172.17.60.78 TCP 58 [TCP Retransmission] 59145 → 1234 [PSH, ACK] Seq=3 Ack=1 Win=132096 Len=2 TSval=3278922962 TSecr=974269329
   60  98.712597 172.17.93.68 → 172.17.60.78 TCP 58 [TCP Retransmission] 59145 → 1234 [PSH, ACK] Seq=3 Ack=1 Win=132096 Len=2 TSval=3278931546 TSecr=974269329
   61 107.364014 172.17.93.68 → 172.17.60.78 TCP 58 [TCP Retransmission] 59145 → 1234 [PSH, ACK] Seq=3 Ack=1 Win=132096 Len=2 TSval=3278940130 TSecr=974269329
   62 116.016472 172.17.93.68 → 172.17.60.78 TCP 44 59145 → 1234 [RST, ACK] Seq=5 Ack=1 Win=132096 Len=0
```

<br>

可以看到没有开启 keep-alive 的情况下，当中间网络断开后，client 是没有感知的，然后client 就会不断的重传。

在重传 13 次后，任务该连接已经断开，然后还是发送了 RST 后， 释放了该连接。