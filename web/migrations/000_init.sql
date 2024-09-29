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
    latest_request integer not null default 0, 
    latest_ping integer not null default 0,
    total_requests integer not null default 0,
    total_requests_time integer not null default 0,
    status text not null default "{}", -- {"400": 100, "200": 2000}
    token text,
    online boolean not null default false,
);

create table if not exists sites_messages (
    id integer primary key not null,
    site integer not null references sites(id) on delete cascade,
    timestamp integer not null,
    text text not null,
    tag text not null
);
