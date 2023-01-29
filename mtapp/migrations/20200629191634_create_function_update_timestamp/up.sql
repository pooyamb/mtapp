CREATE OR REPLACE FUNCTION update_timestamp() RETURNS TRIGGER AS $$
BEGIN
   IF row(NEW.*) IS DISTINCT FROM row(OLD.*) THEN
      NEW.updated_at = now();
      RETURN NEW;
   ELSE
      RETURN OLD;
   END IF;
END;
$$ LANGUAGE 'plpgsql';

-- Or via hstore to get dynamic field names
-- CREATE EXTENSION IF NOT EXISTS hstore;
-- CREATE OR REPLACE FUNCTION update_timestamp() RETURNS TRIGGER AS $$
-- DECLARE
--     _col_value timestamp;
--     _col_name  text := quote_ident(TG_ARGV[0]);
-- BEGIN
--     IF row(NEW.*) IS DISTINCT FROM row(OLD.*) THEN
--         NEW := NEW #= hstore(TG_ARGV[0], 'now()');
--         RETURN NEW;
--     ELSE
--         RETURN OLD;
--     END IF;
-- END;
-- $$ LANGUAGE 'plpgsql';
