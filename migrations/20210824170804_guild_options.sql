-- Add migration script here
CREATE TABLE guild_options
(
	uid bigserial NOT NULL,
	guildid int8 NOT NULL,
	data jsonb NOT NULL,
	key varchar(64) NOT NULL,
    CONSTRAINT guild_options_pk PRIMARY KEY (uid)
);