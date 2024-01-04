# Compile a C program to lc3:
> sudo docker build ./ -t lc3

> docker run -v .:/root/src -ti lc3

> ~/src/lc3-tools# lcc hello.c -o hello_world