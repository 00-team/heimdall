create table if not exists deploys (
    id integer primary key not null,
    repo text not null, -- simurgh
    actor text not null, -- github
    sender text, -- 007, sadra, ...
    begin integer not null default 0,
    finish integer not null default 0,
    status integer not null default 0 -- pending, running, failed, success
);
