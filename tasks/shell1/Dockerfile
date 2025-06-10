FROM debian:12-slim
RUN apt -y update
RUN apt -y install ca-certificates
RUN apt -y install screen

COPY /tty-share.bin /usr/local/bin/tty-share.bin
COPY /flag /flag
RUN chown 1000 /flag
RUN chmod 000 /flag
RUN mkdir /home/user
RUN chown 1000 /home/user
ENV HOME=/home/user
ENV SHELL=/usr/bin/bash
RUN chmod -x /bin/chmod
USER 1000:1000
ENTRYPOINT ["screen", "/usr/local/bin/tty-share.bin", "-public", "-no-wait"]
