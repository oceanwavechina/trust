
# TCP 的 keep-alive 机制

关于 TCP 的 keep-alive 机制，在 rfc1122 中有相应的要求。因为人们对于把 keep-alive 是否应该放到 TCP 里边有争议，所以这个机制不是 **MUST** 包含的。如果实现的话，默认也是要求关闭。 

<br>

## 1. keep-alive 的实现要求
----
<br>

keep-alive 实现的机制如下：

1. keep-alive 包的发送，只能在连接有 **一段时间** 没有 **收到** 包。

2. **一段时间** 要求最少不低于 2 个小时。

3. keep-alive 的探测包不能包含数据。

4. keep-alive 机制建议只在 server 端开启。为了释放连接已经断开的 client 资源。

需要注意的是，如果 keep-alive 失败，并不能断定这个连接一定不可用。


<br>

## 2. keep-alive 在不同场景下的表现
----
<br>

keep-alive 在进行探测时可能会发生 4 种情况： 正常， 对端崩溃， 对端重启， 中间网络异常。

在对比 对端重启 和 中间网络异常 时， 其实更能深入理解： tcp 是端对端的状态协议。什么意思？

1. tcp 连接的两端分别保存了对方的状态信息。

2. 但是它并不知道数据在中间传递时转了几个弯

所以如果没有 keep-alive 机制，当中间网异常时，tcp 连接是无法知道的。

而 keep-alive 机制其实只是在检测两中情况：

1. 对端是否还保存这我们的连接状态信息 （关机和重启后所保存的状态就都没有了）

2. 中间的路是否还能够走通。


TODO: 每种场景下的实例。



liuyanan@liuyanan-K42 ~> sudo tshark -i tun0 -Y "tcp.port==1234"
Running as user "root" and group "root". This could be dangerous.
Capturing on 'tun0'
    1 0.000000000 172.17.93.20 → 172.17.60.78 TCP 60 49592 → 1234 [SYN] Seq=0 Win=65280 Len=0 MSS=1360 SACK_PERM=1 TSval=2484013100 TSecr=0 WS=128
    2 0.170538935 172.17.60.78 → 172.17.93.20 TCP 60 1234 → 49592 [SYN, ACK] Seq=0 Ack=1 Win=14480 Len=0 MSS=1460 SACK_PERM=1 TSval=780600222 TSecr=2484013100 WS=128
    3 0.170603678 172.17.93.20 → 172.17.60.78 TCP 52 49592 → 1234 [ACK] Seq=1 Ack=1 Win=65280 Len=0 TSval=2484013270 TSecr=780600222
    4 4.440292861 172.17.93.20 → 172.17.60.78 TCP 54 49592 → 1234 [PSH, ACK] Seq=1 Ack=1 Win=65280 Len=2 TSval=2484017540 TSecr=780600222
    5 4.588421405 172.17.60.78 → 172.17.93.20 TCP 52 1234 → 49592 [ACK] Seq=1 Ack=3 Win=14592 Len=0 TSval=780604641 TSecr=2484017540


    9 11.855702321 172.17.93.20 → 172.17.60.78 TCP 54 49592 → 1234 [PSH, ACK] Seq=3 Ack=1 Win=65280 Len=2 TSval=2484024956 TSecr=780604641
   10 11.961448799 172.17.60.78 → 172.17.93.20 TCP 52 1234 → 49592 [ACK] Seq=1 Ack=5 Win=14592 Len=0 TSval=780612051 TSecr=2484024956


   16 40.565525268 172.17.93.20 → 172.17.60.78 TCP 54 49592 → 1234 [PSH, ACK] Seq=5 Ack=1 Win=65280 Len=2 TSval=2484053665 TSecr=780612051
   21 41.053637827 172.17.93.20 → 172.17.60.78 TCP 54 [TCP Retransmission] 49592 → 1234 [PSH, ACK] Seq=5 Ack=1 Win=65280 Len=2 TSval=2484054154 TSecr=780612051
   23 41.565670398 172.17.93.20 → 172.17.60.78 TCP 54 [TCP Retransmission] 49592 → 1234 [PSH, ACK] Seq=5 Ack=1 Win=65280 Len=2 TSval=2484054666 TSecr=780612051
   26 42.557642562 172.17.93.20 → 172.17.60.78 TCP 54 [TCP Retransmission] 49592 → 1234 [PSH, ACK] Seq=5 Ack=1 Win=65280 Len=2 TSval=2484055658 TSecr=780612051
   29 44.509643651 172.17.93.20 → 172.17.60.78 TCP 54 [TCP Retransmission] 49592 → 1234 [PSH, ACK] Seq=5 Ack=1 Win=65280 Len=2 TSval=2484057610 TSecr=780612051
   30 48.637659930 172.17.93.20 → 172.17.60.78 TCP 54 [TCP Retransmission] 49592 → 1234 [PSH, ACK] Seq=5 Ack=1 Win=65280 Len=2 TSval=2484061738 TSecr=780612051
   31 56.577634559 172.17.93.20 → 172.17.60.78 TCP 54 [TCP Retransmission] 49592 → 1234 [PSH, ACK] Seq=5 Ack=1 Win=65280 Len=2 TSval=2484069678 TSecr=780612051
   32 72.189645906 172.17.93.20 → 172.17.60.78 TCP 54 [TCP Retransmission] 49592 → 1234 [PSH, ACK] Seq=5 Ack=1 Win=65280 Len=2 TSval=2484085290 TSecr=780612051
   33 105.213715587 172.17.93.20 → 172.17.60.78 TCP 54 [TCP Retransmission] 49592 → 1234 [PSH, ACK] Seq=5 Ack=1 Win=65280 Len=2 TSval=2484118314 TSecr=780612051
   44 168.701667165 172.17.93.20 → 172.17.60.78 TCP 54 [TCP Retransmission] 49592 → 1234 [PSH, ACK] Seq=5 Ack=1 Win=65280 Len=2 TSval=2484181802 TSecr=780612051
   50 289.533661166 172.17.93.20 → 172.17.60.78 TCP 54 [TCP Retransmission] 49592 → 1234 [PSH, ACK] Seq=5 Ack=1 Win=65280 Len=2 TSval=2484302634 TSecr=780612051
   67 410.365726755 172.17.93.20 → 172.17.60.78 TCP 54 [TCP Retransmission] 49592 → 1234 [PSH, ACK] Seq=5 Ack=1 Win=65280 Len=2 TSval=2484423466 TSecr=780612051
   72 531.197720827 172.17.93.20 → 172.17.60.78 TCP 54 [TCP Retransmission] 49592 → 1234 [PSH, ACK] Seq=5 Ack=1 Win=65280 Len=2 TSval=2484544298 TSecr=780612051
   77 652.029700302 172.17.93.20 → 172.17.60.78 TCP 54 [TCP Retransmission] 49592 → 1234 [PSH, ACK] Seq=5 Ack=1 Win=65280 Len=2 TSval=2484665130 TSecr=780612051
   82 772.861733624 172.17.93.20 → 172.17.60.78 TCP 54 [TCP Retransmission] 49592 → 1234 [PSH, ACK] Seq=5 Ack=1 Win=65280 Len=2 TSval=2484785962 TSecr=780612051
   88 893.693731817 172.17.93.20 → 172.17.60.78 TCP 54 [TCP Retransmission] 49592 → 1234 [PSH, ACK] Seq=5 Ack=1 Win=65280 Len=2 TSval=2484906794 TSecr=780612051

