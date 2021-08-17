-- Add migration script here
CREATE TABLE public.user_points (
	uid bigserial NOT NULL,
	userid int8 NOT NULL,
	guildid int8 NOT NULL,
	points int8 NOT NULL
);
