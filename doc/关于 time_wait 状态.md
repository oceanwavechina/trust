# 关于 time_wait 状态

## 1. 怎么产生的

首先从头字面上看 time_wait， 它等待的是 "超时时间"， 而不是事件。

>这个超时时间是 2MSL, 即 2 倍的 Maximum Segment Lifetime， 等待2MSL时间主要目的是怕最后一个ACK包对方没收到，那么对方在超时后将重发第三次握手的FIN包，主动关闭端接到重发的FIN包后可以再发一个ACK应答包。在TIME_WAIT状态时两端的端口不能使用，要等到2MSL时间结束才可继续使用。当连接处于2MSL等待阶段时任何迟到的报文段都将被丢弃。

```
Maximum Segment Lifetime

the time a TCP segment can exist in the internetwork system.  

Arbitrarily defined to be 2 minutes.
```

要理解这个东西，先要明白tcp的关闭分为两种：
1. 主动关闭
2. 被动关闭

而 time_wait 就是发生在 **主动关闭** 情况下，根据 rfc793 的 tcp 状态变迁图，可以知道，当主动关闭时，会有如下的状态变迁路径（以当前端为第一视角）

```
ESTABLISHED  
        >>>>    close: send FIN     >>>>   FIN_WAIT_1  

        >>>>    recv ACK            >>>>   FIN_WAIT_2

        >>>>    recv FIN, send ACK  >>>>   TIME_WAIT

        >>>>        wait 2MSL       >>>>   CLOSE
```        

从上边的流程可以知道 主动发起关闭的一方，主要是在等对方的回应。并且最后要回复对方ACK，需要注意的是这个ACK本身并不会得到对方的确认，
也就是不能确保这个ACK一定会被对方收到，为了保证这个连接（四元组）完整的关闭，只有等待一定时间。

问题来了，这个一段时间是怎么确定的。因为 TCP 传输的可靠性依赖超时重传，所以要假设第一次没有传成功，然后重传一次
所以我们假设的最大超时时间就是传输两次的时间，


## 2. 通过对比来理解 MSL

* MSL
  
    他是任何报文在网络上存在的最长时间，超过这个时间报文将被丢弃

* TTL
  
    IP 头中有一个TTL域，TTL是 time to live的缩写，中文可以译为“生存时间”，这个生存时间是由源主机设置初始值但不是存的具体时间，而是存储了一个ip数据报可以经过的最大路由数，每经 过一个处理他的路由器此值就减1，当此值为0则数据报将被丢弃，同时发送ICMP报文通知源主机。

* RTT

    RTT是客户到服务器往返所花时间（round-trip time，简称RTT），TCP含有动态估算RTT的算法。TCP还持续估算一个给定连接的RTT，这是因为RTT受网络传输拥塞程序的变化而变化


## 3. 如何解决服务端大量 time_wait 导致服务不可用的状态

首先 time_wait 的主要作用是防止新建立的连接受到之前数据包的干扰，因为如果没有 time_wait，快速建立的同样的连接(四元组)，有可能接收到之前连接发出的数据。 所以 time_wait 是为了，新建连接时，该链路上旧的数据包已经超时被丢弃了， 所以time_wait本身是有重要意义的

另外产生 time_wait 的那端，一定是主动关闭连接的一端。这样我们可以让client端来主动关闭，把time_wait的压力从server端转移出去

还有算是暴力处理：
    
   1. 设置 SO_LINGER 为0， 也就是当我们主动关闭 TCP 连接时，直接丢弃缓冲区中的信息，然后直接发送RST 消息，而不走正常的关闭流程
   
   2. 使用 net.ipv4.tcp_tw_reuse 选项，通过引入时间戳来重用处于 time_wait 的端口
       By enabling net.ipv4.tcp_tw_reuse, Linux will reuse an existing connection in the TIME-WAIT state for a new **outgoing connection** if the new timestamp is strictly bigger than the most recent timestamp recorded for the previous connection: an outgoing connection in the TIME-WAIT state can be reused after just one second.
       
       注意这里对于 server 端来说， 这个选项对于 incoming connections 没有用处






https://vincent.bernat.ch/en/blog/2014-tcp-time-wait-state-linux#netipv4tcp_tw_reuse