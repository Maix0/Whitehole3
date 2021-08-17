-- Add migration script here

CREATE TABLE role_points(
	uid bigserial NOT NULL,
	roleid int8 NOT NULL,
	guildid int8 NOT NULL,
	points int8 NOT NULL,
	CONSTRAINT role_points_pk PRIMARY KEY (uid)
);

