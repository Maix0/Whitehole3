-- Add migration script here
CREATE TABLE "user_permission" (
  "uid" serial8,
  "guildid" int8 NOT NULL,
  "userid" int8 NOT NULL,
  "ids" TEXT[] NOT NULL,
  PRIMARY KEY ("uid")
);

