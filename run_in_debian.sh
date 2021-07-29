#!/bin/bash

sh copy_to_debian.sh
ssh liuyanan@192.168.43.230 'cd /home/liuyanan/lab/rust_programming/trust; bash run.sh'
