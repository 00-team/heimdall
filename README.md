# heimdall

watchdog for websites

## Roadmap

### version 1

1. [ ] simple
1. [ ] do not store individual requests\
        thus there is no pre date data
1. [ ] average request processing time
1. [ ] error rate (e.g. pre 100 `200` requests there are 5 `505` responses. 5% error rate)
1. [ ] average request pre day
1. [ ] average request size?

## Usage

put this in your websites nginx config (requries upstream)

```nginx
server {
    ...

    location / {
        access_log syslog:server=unix:/usr/share/nginx/socks/heimdall.dog.sock,tag=H,nohostname heimdall;

        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Client-Ip $remote_addr;
        proxy_set_header Host $http_host;
        proxy_redirect off;

        proxy_pass http://upstream_unix_sock;
    }

    ...
}
```

put this in `/etc/nginx/nginx.conf`

```nginx
http {
    ...

    include /path-to/heimdall/config/format.conf;

    ...
}
```
