# tcp 协议的实现

## 1. 运行步骤
cargo r --release
sudo setcap cap_net_admin=eip /home/liuyanan/my_work/rust_programming.git/trust/target/release/trust

./target/release/trust

ip addr set 192.168.0.1

sudo ip addr add 192.168.0.1/24 dev tun0


sudo ip link set up dev tun0

ping -I tun0 192.168.0.2


命令行版本的wireshark

sudo tshark -i tun0



pgrep -af python


carte.io   找包


http://www.iana.org/assignments/protocol-numbers/protocol-numbers.xhtml

nc 192.168.0.2 9000
> 192.168.0.1 → 192.168.0.2 40b of tcp to port 80

tshark -i tun0

3:
    2:56:00
