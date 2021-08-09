#!/usr/bin/env python3

import socket

HOST = '172.17.79.12'  # The server's hostname or IP address
PORT = 2000        # The port used by the server

with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
    s.connect((HOST, PORT))
    s.sendall(b'Hello, world')
    data = s.recv(1024)

    while 1:
        pass

print('Received', repr(data))

