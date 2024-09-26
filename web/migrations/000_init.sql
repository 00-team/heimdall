create table if not exists users (
    id integer primary key not null,
    phone text not null,
    name text not null default "",
    token text,
    admin boolean not null default false
);
