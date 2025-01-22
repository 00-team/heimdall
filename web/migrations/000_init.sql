create table if not exists users (
    id integer primary key not null,
    phone text not null unique,
    name text not null default "",
    token text,
    admin boolean not null default false
);

create table if not exists sites (
    id integer primary key not null,
    name text not null unique,
    timestamp integer not null default 0,
    latest_request integer not null default 0, 
    latest_ping integer not null default 0,
    total_requests integer not null default 0,
    total_requests_time integer not null default 0,
    requests_max_time integer not null default 0,
    requests_min_time integer not null default 0,
    status text not null default "{}", -- {"400": 100, "200": 2000}
    token text,
    online boolean not null default false,
    latest_message_timestamp integer not null default 0,
    latest_dump_timestamp integer not null default 0
);

create table if not exists sites_messages (
    id integer primary key not null,
    site integer not null references sites(id) on delete cascade,
    timestamp integer not null,
    text text not null,
    tag text not null
);

create table if not exists deploys (
    id integer primary key not null,
    repo text not null, -- simurgh
    actor text not null, -- github
    sender text, -- 007, sadra, ...
    begin integer not null default 0,
    finish integer not null default 0,
    status integer not null default 0, -- pending, running, failed, success
    stdout text,
    stderr text
);
