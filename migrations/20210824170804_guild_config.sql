-- Add migration script here
CREATE TABLE guild_config
(
	uid bigserial NOT NULL,
	guildid int8 NOT NULL,
	data jsonb NOT NULL,
	key varchar(64) NOT NULL,
    CONSTRAINT guild_config_pk PRIMARY KEY (uid)
);

CREATE OR REPLACE FUNCTION get_config (guildid_in int8, key_in varchar) RETURNS jsonb AS $$
    DECLARE d JSONB;
    BEGIN
        SELECT pg_advisory_lock(uid) FROM guild_config WHERE guildid = guildid_in AND key = key_in;

        SELECT data INTO d FROM guild_config WHERE guildid = guildid_in AND key = key_in;
        RETURN d;
    END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION set_config (guildid_in int8, key_in varchar, data_in jsonb) RETURNS void AS $$
    BEGIN
        UPDATE guild_config SET data = data_in WHERE guildid = guildid_in AND key = key_in;
        SELECT pg_advisory_unlock(uid) FROM guild_config WHERE guildid = guildid_in AND key = key_in;
    END;
$$ LANGUAGE plpgsql;
