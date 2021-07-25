add_commands!(Music, (clear, join, pause, play, queue, resume, skip), ());

add_commands!(MusicPriv, (move_cmd, remove, leave), (music_manage));

check_permission!(MUSIC_MANAGE_CHECK, "music.manage");
