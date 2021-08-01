-- Add migration script here

CREATE TABLE public.user_playlist (
	uid bigserial NOT NULL,
	userid int8 NOT NULL,
	guildid int8 NOT NULL,
	items text[] NOT NULL,
	"name" varchar(32) NOT NULL,
	CONSTRAINT user_playlist_pk PRIMARY KEY (uid)
);
