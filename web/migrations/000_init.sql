create table if not exists users (
    id integer primary key not null,
    phone text not null unique,
    name text not null default "",
    token text,
    admin boolean not null default false
);

create table if not exists projects (
    slug text primary key not null,
    latest_request integer not null default 0, 
    total_requests integer not null default 0,
    token text
);
