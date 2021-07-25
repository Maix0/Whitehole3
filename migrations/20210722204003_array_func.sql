-- Add migration script here
create or replace function array_diff(array1 anyarray, array2 anyarray)
returns anyarray language sql immutable as $$
    select coalesce(array_agg(elem), '{}')
    from unnest(array1) elem
    where elem <> all(array2)
$$;