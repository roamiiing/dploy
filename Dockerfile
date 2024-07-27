FROM alpine:latest

EXPOSE 8000

CMD [ "/bin/sh", "-c", "echo 'Hello, World!' && sleep infinity" ]
