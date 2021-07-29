# Nagle算法和滑动窗口算法冲突了？

tcp 协议实现中同时实现了 Nalge 算法 和 滑动窗口 算法。乍一看，这个两个算法好像是冲突的

* 前者要求网络上只能有一个未确认的分组，而后者允许在没有收到确认的情况下发送多个分组。

* 既然 Nalge 算法要求了网络上只能有一个为确认的分组，那 TCP 分组报文失序又是从何而来?

本文主要基于 RFC896 和 RFC793

<br>

## 1. Nagle 算法
----

先看看 Nagle 算法是怎么定义的，它要解决什么问题

> Nagla 算法: 要求一个tcp连接上最多只能有一个未被确认的未完成的小分组
>
> 是为了减少广域网的小分组数目，从而减小网络拥塞的出现

Nagle 算法出现在 rfc896 中，目的是解决互联网中的拥塞问题。我们就先看下，拥塞问题产生的背景，是因为复杂的网络环境，根据 rfc896 中的描述：

1. 当时在福特汽车私有的 TCP 网络中，因为不同部分的网络连接到一起，导致数据在不同网络段中传输速率相差很大，从 1200 to 10000000 bps

2. 在高负载的网路中，当转发节点出现拥塞时，数据包的往返时间 和 网络上数据包的数量 都会增加

3. 如果某个时间点，网络上数据包激增， 虽然 TCP 实现中会有根据数包的往返时间来确定数据重传的间隔，但是这种瞬时激增导致 TCP 来不及更新重传间隔。那 TCP 会触发数据包重传，给本来已经拥塞的网络上塞入更过的数据包。算是恶性循环，网络会进一步恶化

<br>

### 1.1 小包问题

在高负载的网络环境中，大量的小包就成了问题，因为高负载的网络中，数据可能丢失，导致重传，给本就拥挤的网络雪上加霜，导致 TCP 连接的吞吐量大大下降

rfc896 之前已经出现了小包问题的解决办法，就是把小包尽可能大延时发送，以减少网络上的数据包的数量。也就是尽量合并小包发送，但是延时多长时间是个问题，延时太小不足以合并多个小包，延时太长就会造成用户体验问题。而且网络环境复杂，当低负载的网络中多个小包，并不会出现问题。而高负载传输率较低的网络中这个固定的延时可能起不到什么作用。问题没有得到很好的解决的原因，就是这个办法不是动态适应的，没法适用于各种复杂的网络环境。

<br>

### 1.2 Nagle 算法流程

于是才有了 Nagle 算法：
```
The solution is to inhibit the sending of new TCP  segments  when new  outgoing  data  arrives  from  the  user  if  any previously transmitted data on the connection remains unacknowledged.

This inhibition  is  to be unconditional; no timers, tests for size of data received, or other conditions are required
```

然后看看为什么这个算法是可行的
```
When a user process writes to a TCP connection, TCP receives some data. It may hold that data for future sending or may send a packet immediately.  If it refrains from sending now, it will typically send the data later when an incoming packet arrives and changes the state of the system.  

The state changes in one of two ways;  
    
    * the incoming packet acknowledges old data the distant host has received

    * or announces the availability of  buffer  space  in the  distant  host  for  new  data.  
        (This last is referred to as "updating the window").    

Each time data arrives on a connection, TCP must reexamine its current state and perhaps send some packets out.  Thus, when we omit sending data on arrival from the user,  we are simply  deferring its transmission until the next message arrives from the distant host.   A message must always arrive soon unless the connection was previously idle or communications with the other end have been lost.
```

这里有算法起始步骤需要注意，

* 就是连接开始建立的时候，因为我们没有 ACK 需要等待， 所以是先发一个包，然后等待 ACK。

* 当第一个包的ACK，回来后，我们有两个选择，
  1. 当 ACK 里边不包含 update window 的标记，我们就按照小包的逻辑。再次发送一个包，然后等待
  2. 当 ACK 里边包含了 peer 端可以容纳更多的树 update window 标记，我们就进入了滑动窗口的逻辑

所以到这里，就会发下 Nagle 算法，是在滑动窗口的基础上加了一个限制。 相当于把原有 TCP 启动时的流程(第一个包)做了限制，根据第一个包的 ACK 做进一步的判断。

<br>

### 1.3 ICMP 源端抑制 消息的处理

这里主要分析了当我们收到 ICMP 的 Source Quench 消息后该如何处理。

首先 ICMP 的源端抑制消息 是网关 IP 层发出来的消息，但是我们最好在 TCP 层也接受处理，因为可以知道我们对 TCP 的行为进行改变。

比如如果 TCP 不知道这个消息，还会继续触发超时重传如果，还不成功则认为连接已经断开，而实际上连接还在。所以我们在 TCP 层要做的就是把发送窗口置为 0， 等待更多的 ACK 到达后，在慢慢增大这个窗口。这样可以减少数据重传，以减少本就拥挤的网络的压力

> In each case we would suggest that new traffic should be throttled but acknowledges  should  be  treated normally.

需要注意的是，我们只对新的数据进行限制，但是不会限制 ACK 类型的消息，这样就不会导致连接断开

可以看到，ICMP Source Quench 的处理原则，也是尽量减少 弱网/高负责 场景下的带宽占用。因为一旦网络拥堵，接下来的 retransmit 应该都是无效的。

同样的思路，推广到我们上层的业务中，比如如果某个时刻，突然的网络请求不可达，我们一般的处理做法就是重试，但是如果因为上游并发问题导致不可访问，频繁的重试，不仅不会解决问题，还会造成雪上加霜。所以我们直接返回失败，或是等待一定时间在试。

<br>

## 2. 滑动窗口
----

什么是窗口

窗口是有接收方决定的，也就是接收方在 ACK 中通知发送方还可以发送多少数据

```
Flow Control：
    TCP provides a means for the receiver to govern the amount of data
    sent by the sender.  This is achieved by returning a "window" with
    every ACK indicating a range of acceptable sequence numbers beyond
    the last segment successfully received.  The window indicates an
    allowed number of octets that the sender may transmit before
    receiving further permission
```

滑动窗口是 TCP 进行流量控制的一种方式，以确保收发两端在匹配的速率下工作。算是动态进行速率匹配的一种方法

<br>

## 3. 二者为什么不冲突
----

从上边的分析可以看到，二者的实现是融合在一起的。
* 滑动窗口则是为了解决收发两端的 发送/接收 速率不匹配的问题，它的关注点是收发两端。假设的是中间链路特别好。
* Nagle 主要解决了在 高负载链路的 或是 不稳定链路上，数据拥塞的问题，注意是它关注的的链路本身的问题，而不是两端

从发展的时间上看
* 滑动窗口在TCP设计之初就引入了。
* Nagle 拥塞控制是在 TCP 后来的运行过程中产生拥塞问题之后再找到的一种解决办法
