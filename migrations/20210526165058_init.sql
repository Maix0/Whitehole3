CREATE TABLE "points" (
  "uid" serial8,
  "guildid" int8 NOT NULL,
  "userid" int8 NOT NULL,
  "points" int8 NOT NULL,
  "last_valid" timestamp,
  PRIMARY KEY ("uid")
);
COMMENT ON COLUMN "points"."uid" IS 'Unique User ID';
COMMENT ON COLUMN "points"."guildid" IS 'Guild ID';
COMMENT ON COLUMN "points"."userid" IS 'User ID';
COMMENT ON COLUMN "points"."points" IS 'Points';
COMMENT ON COLUMN "points"."last_valid" IS 'Time before points valid';

CREATE TABLE "User" (
  "uid" serial8,
  "guildid" int8 NOT NULL,
  "userid" int8 NOT NULL,
  PRIMARY KEY ("uid")
);
COMMENT ON COLUMN "User"."uid" IS 'User Unique ID';
COMMENT ON COLUMN "User"."guildid" IS 'User Guild ID';
COMMENT ON COLUMN "User"."userid" IS 'User ID';

ALTER TABLE "User" ADD CONSTRAINT "fk_User_points_1" FOREIGN KEY ("uid") REFERENCES "points" ("uid");