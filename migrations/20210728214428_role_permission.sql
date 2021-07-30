-- Add migration script here

CREATE TABLE public.role_permission (
	uid bigserial NOT NULL,
	guildid int8 NOT NULL,
	roleid int8 NOT NULL,
	ids text[] NOT NULL,
	CONSTRAINT role_permission_pk PRIMARY KEY (uid)
);
