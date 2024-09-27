# heimdall

watchdog for websites

## Roadmap

### Version 1

1. simple
1. do not store individual requests\
   thus there is no pre date data
1. [ ] average request processing time
1. [ ] error rate (e.g. pre 100 `200` requests there are 5 `505` responses. 5% error rate)
1. [ ] average request pre day
1. [ ] average request size?
1. [ ] message queue for each project. for verify code logs and ...

### Future

1. request pre country / show geo ip on map
1. total request for each uri
1. most used queries: (e.g. ?page=1 ?filter=...)
1. request pre date / total request from A to B months or days
1. estimate the future requests
1. show users using cookie_authorization
1. bounce rate\
   users requests the home page `/` then goes to dashboard `/dash/`\
   then goes to `/dash/orders/`. keep track of time between requests\
   and the route that users take
1. top page
1. returning visitors
1. device type
1. landing page conversion rate\
   when users comes to the home pages. does it gose to any other page?

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
