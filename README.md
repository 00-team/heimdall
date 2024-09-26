# heimdall

watchdog for websites

## Usage

put this in your websites nginx config (requries upstream)

```conf
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

```conf
http {
    ...

    include /path-to/heimdall/config/format.conf;

    ...
}
```
