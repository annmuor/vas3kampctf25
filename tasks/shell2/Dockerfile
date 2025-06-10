FROM debian:12-slim
RUN apt -y update
RUN apt -y install ca-certificates
RUN apt -y install screen

COPY /tty-share.bin /usr/local/bin/tty-share.bin
COPY /flag /flag
RUN chown 1000 /flag
RUN mkdir /home/user
RUN chown 1000 /home/user
RUN find /usr/bin/ -type f|grep -vP 'ls|bash|screen'|xargs rm
ENV SHELL=/usr/bin/bash
ENV HOME=/home/user
USER 1000:1000
ENTRYPOINT ["screen", "/usr/local/bin/tty-share.bin", "-public", "-no-wait"]
