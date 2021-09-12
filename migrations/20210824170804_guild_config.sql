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
        PERFORM pg_advisory_lock(uid) FROM guild_config WHERE guildid = guildid_in AND key = key_in;
        SELECT data INTO d FROM guild_config WHERE guildid = guildid_in AND key = key_in;
        RETURN d;
    END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION set_config (guildid_in int8, key_in varchar, data_in jsonb) RETURNS void AS $$
    BEGIN
        IF EXISTS (SELECT * FROM guild_config WHERE guildid = guildid_in AND key = key_in) THEN
            UPDATE guild_config SET data = data_in WHERE guildid = guildid_in AND key = key_in;
        ELSE
            INSERT INTO guild_config (guildid, data, key) VALUES (guildid_in,data_in, key_in);
        END IF;
        PERFORM pg_advisory_unlock(uid) FROM guild_config WHERE guildid = guildid_in AND key = key_in;
    END;
$$ LANGUAGE plpgsql;
