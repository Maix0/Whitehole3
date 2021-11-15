use crate::{CacheHttp, Data};
use rocket::{
    http::{ContentType, Status},
    Route, State,
};
use serenity::model::id::{GuildId, RoleId, UserId};

#[inline(always)]
fn handle_error<T: std::error::Error>(x: T) -> (Status, String) {
    (Status::InternalServerError, x.to_string())
}

#[get("/leaderboard/<guildid>?<page>")]
#[allow(clippy::format_in_format_args)]
async fn get_leaderbord(
    cachehttp: &State<CacheHttp>,
    data: &State<Data>,
    page: Option<u16>,
    guildid: u64,
) -> Result<(ContentType, Vec<u8>), (Status, String)> {
    let lock = data.read().await;
    let db = lock.get::<wh_database::shared::DatabaseKey>().unwrap();
    let page = page.unwrap_or(1).max(1);
    let res = query!(
        "
        SELECT userid, points FROM user_points
        WHERE guildid = $1::int8
        GROUP BY points, userid
        ORDER BY points DESC
        LIMIT 10 OFFSET $2
        ",
        wh_database::shared::Id(guildid) as _,
        ((page - 1) * 10) as i32
    )
    .fetch_all(db)
    .await
    .map_err(handle_error)?;
    let mut pairs: Vec<(u64, i64)> = Vec::with_capacity(res.len());
    for re in res {
        pairs.push((unsafe { std::mem::transmute(re.userid) }, re.points));
    }
    let mut pairs_iter = pairs.iter().enumerate();

    let res = query!(
        "
        SELECT points, roleid FROM role_points 
        WHERE guildid = $1::int8
        ORDER BY points DESC
        ",
        wh_database::shared::Id(guildid) as _
    )
    .fetch_all(db)
    .await
    .map_err(handle_error)?;

    let mut role_pairs = Vec::<(u64, i64, u32)>::with_capacity(res.len());

    let roles = GuildId(guildid)
        .roles(&cachehttp.http)
        .await
        .map_err(handle_error)?;
    for re in res {
        let (roleid, points) = (unsafe { std::mem::transmute(re.roleid) }, re.points);
        let color = roles
            .get(&RoleId(roleid))
            .map(|r| r.colour.0)
            .unwrap_or_else(|| {
                debug!("else: {}", roleid);
                0xFFFFFF
            });
        role_pairs.push((roleid, points, color));
    }
    let leaderboard = format!(
        include_str!("imgs/leaderboard.svg"),
        header = format!(
            include_str!("imgs/header.svg"),
            guid_name = GuildId(guildid)
                .name(&cachehttp.cache)
                .await
                .unwrap_or_else(|| String::from("Leaderboard"))
        ),
        item1 = generate_rank_item(page, &mut pairs_iter, cachehttp.inner(), &role_pairs).await,
        item2 = generate_rank_item(page, &mut pairs_iter, cachehttp.inner(), &role_pairs).await,
        item3 = generate_rank_item(page, &mut pairs_iter, cachehttp.inner(), &role_pairs).await,
        item4 = generate_rank_item(page, &mut pairs_iter, cachehttp.inner(), &role_pairs).await,
        item5 = generate_rank_item(page, &mut pairs_iter, cachehttp.inner(), &role_pairs).await,
        item6 = generate_rank_item(page, &mut pairs_iter, cachehttp.inner(), &role_pairs).await,
        item7 = generate_rank_item(page, &mut pairs_iter, cachehttp.inner(), &role_pairs).await,
        item8 = generate_rank_item(page, &mut pairs_iter, cachehttp.inner(), &role_pairs).await,
        item9 = generate_rank_item(page, &mut pairs_iter, cachehttp.inner(), &role_pairs).await,
        item10 = generate_rank_item(page, &mut pairs_iter, cachehttp.inner(), &role_pairs).await,
    );
    let mut fontdb = usvg::fontdb::Database::new();

    fontdb.load_fonts_dir("wh_webserver/font");
    fontdb.load_system_fonts();

    let svg = usvg::Tree::from_str(
        leaderboard.as_str(),
        &usvg::Options {
            fontdb,
            ..Default::default()
        }
        .to_ref(),
    )
    .map_err(handle_error)?;

    let mut pixmap = tiny_skia::Pixmap::new(800, 600).unwrap();

    resvg::render(&svg, usvg::FitTo::Original, pixmap.as_mut());
    //Ok((ContentType::SVG, leaderboard.into_bytes()))
    Ok((ContentType::PNG, pixmap.encode_png().map_err(handle_error)?))
}
async fn generate_rank_item<'a>(
    page: u16,
    iter: &mut impl Iterator<Item = (usize, &'a (u64, i64))>,
    http: &CacheHttp,
    roles: &[(u64, i64, u32)],
) -> String {
    if let Some((sub_rank, pair)) = iter.next() {
        format!(
            include_str!("imgs/rankitem.svg"),
            rank = (sub_rank + (page - 1) as usize * 10) + 1,
            rank_font_size = if (sub_rank + (page - 1) as usize * 10) + 1 < 10 {
                48
            } else if (sub_rank + (page - 1) as usize * 10) + 1 < 100 {
                36
            } else {
                24
            },
            rank_y = if (sub_rank + (page - 1) as usize * 10) + 1 < 10 {
                35.34
            } else if (sub_rank + (page - 1) as usize * 10) + 1 < 100 {
                31.38
            } else {
                27.42
            },
            rank_x = if (sub_rank + (page - 1) as usize * 10) + 1 < 10 {
                37.0
            } else {
                30.5
            },
            points = pair.1,
            username = UserId(pair.0)
                .to_user(http)
                .await
                .map(|u| u.tag())
                .unwrap_or_else(|_| pair.0.to_string()),
            circle_color = match (page, sub_rank) {
                (1, 0) => "ffdb58",
                (1, 1) => "aaa9ad",
                (1, 2) => "cd7f32",
                _ => "ffffff",
            },
            y = match sub_rank {
                0 => "60",
                1 => "115",
                2 => "170",
                3 => "225",
                4 => "280",
                5 => "335",
                6 => "390",
                7 => "445",
                8 => "500",
                9 => "555",
                _ => "10000",
            },
            role_color = {
                format!(
                    "{:06X}",
                    roles
                        .iter()
                        .filter(|x| x.1 <= pair.1)
                        .map(|x| x.2)
                        .next()
                        .unwrap_or(0xFFFFFF)
                )
            }
        )
    } else {
        String::new()
    }
}

async fn generate_queue_item(
    http: &CacheHttp,
    queue_pos: u16,
    data: Option<&wh_music::shared::Song>,
) -> String {
    if let Some(data) = data {
        let seconds = data.duration.as_secs() % 60;
        let minutes = (data.duration.as_secs() / 60) % 60;
        let hours = (data.duration.as_secs() / 60) / 60;

        let username = UserId(data.added_by)
            .to_user(http)
            .await
            .map(|u| u.tag())
            .unwrap_or_else(|_| data.added_by.to_string());

        format!(
            include_str!("imgs/queue_item.svg"),
            title = &data.title,
            y = (queue_pos % 10) * 80 + 28,
            queue_rank = queue_pos,
            duration = format_args!("{:02}:{:02}:{:02}", hours, minutes, seconds),
            loop_state = {
                match data.loop_num {
                    wh_music::shared::LoopState::None => "1".to_string(),
                    wh_music::shared::LoopState::Finite(num) => num.to_string(),
                    wh_music::shared::LoopState::Infinite => "∞".to_string(),
                }
            },
            added_by = username,
            rank_font = match queue_pos {
                0..=99 => "36",
                _ => "24",
            },
            y_rank = match queue_pos {
                0..=99 => "31.38",
                _ => "27.42",
            },
            x_rank = match queue_pos {
                0..=9 => "10.19",
                10..=99 => "0",
                100.. => "0",
            }
        )
    } else {
        String::new()
    }
}

async fn generate_queue_nowplaying(http: &CacheHttp, data: wh_music::shared::NowPlaying) -> String {
    let play_seconds = data.time_in.as_secs() % 60;
    let play_minutes = (data.time_in.as_secs() / 60) % 60;
    let play_hours = (data.time_in.as_secs() / 60) / 60;

    let seconds = data.song.duration.as_secs() % 60;
    let minutes = (data.song.duration.as_secs() / 60) % 60;
    let hours = (data.song.duration.as_secs() / 60) / 60;

    let username = UserId(data.song.added_by)
        .to_user(http)
        .await
        .map(|u| u.tag())
        .unwrap_or_else(|_| data.song.added_by.to_string());
    format!(
        include_str!("imgs/queue_now_playing.svg"),
        total_duration = format_args!("{:02}:{:02}:{:02}", hours, minutes, seconds),
        current_duration =
            format_args!("{:02}:{:02}:{:02}", play_hours, play_minutes, play_seconds),
        title = data.song.title,
        loop_state = {
            match data.song.loop_num {
                wh_music::shared::LoopState::None => "1".to_string(),
                wh_music::shared::LoopState::Finite(num) => num.to_string(),
                wh_music::shared::LoopState::Infinite => "∞".to_string(),
            }
        },
        added_by = username,
    )
}
#[post("/queue/now_playing", data = "<now_playing>")]
async fn get_now_playing(
    cachehttp: &State<CacheHttp>,
    now_playing: rocket::serde::json::Json<wh_music::shared::NowPlaying>,
) -> Result<(ContentType, Vec<u8>), (Status, String)> {
    let now_playing = now_playing.into_inner();
    let mut fontdb = usvg::fontdb::Database::new();

    let now_playing_str = generate_queue_nowplaying(cachehttp, now_playing).await;

    fontdb.load_fonts_dir("wh_webserver/font");
    fontdb.load_system_fonts();

    let svg = usvg::Tree::from_str(
        now_playing_str.as_str(),
        &usvg::Options {
            fontdb,
            ..Default::default()
        }
        .to_ref(),
    )
    .map_err(handle_error)?;

    let mut pixmap = tiny_skia::Pixmap::new(800, 210).unwrap();

    resvg::render(&svg, usvg::FitTo::Original, pixmap.as_mut());
    //Ok((ContentType::SVG, leaderboard.into_bytes()))
    Ok((ContentType::PNG, pixmap.encode_png().map_err(handle_error)?))
}

#[post("/queue/list", data = "<queue_data>")]
async fn get_queue(
    cachehttp: &State<CacheHttp>,
    queue_data: rocket::serde::json::Json<wh_music::shared::QueueRequest>,
) -> Result<(ContentType, Vec<u8>), (Status, String)> {
    let queue_data = queue_data.into_inner();
    let mut fontdb = usvg::fontdb::Database::new();

    let queue = format!(
        include_str!("imgs/queue.svg"),
        queue_item_1 = generate_queue_item(
            cachehttp,
            queue_data.page_number as u16 * 10 + 1,
            queue_data.queue.get(0)
        )
        .await,
        queue_item_2 = generate_queue_item(
            cachehttp,
            queue_data.page_number as u16 * 10 + 2,
            queue_data.queue.get(1)
        )
        .await,
        queue_item_3 = generate_queue_item(
            cachehttp,
            queue_data.page_number as u16 * 10 + 3,
            queue_data.queue.get(2)
        )
        .await,
        queue_item_4 = generate_queue_item(
            cachehttp,
            queue_data.page_number as u16 * 10 + 4,
            queue_data.queue.get(3)
        )
        .await,
        queue_item_5 = generate_queue_item(
            cachehttp,
            queue_data.page_number as u16 * 10 + 5,
            queue_data.queue.get(4)
        )
        .await,
        queue_item_6 = generate_queue_item(
            cachehttp,
            queue_data.page_number as u16 * 10 + 6,
            queue_data.queue.get(5)
        )
        .await,
        queue_item_7 = generate_queue_item(
            cachehttp,
            queue_data.page_number as u16 * 10 + 7,
            queue_data.queue.get(6)
        )
        .await,
        queue_item_8 = generate_queue_item(
            cachehttp,
            queue_data.page_number as u16 * 10 + 8,
            queue_data.queue.get(7)
        )
        .await,
        queue_item_9 = generate_queue_item(
            cachehttp,
            queue_data.page_number as u16 * 10 + 9,
            queue_data.queue.get(8)
        )
        .await,
        queue_item_10 = generate_queue_item(
            cachehttp,
            queue_data.page_number as u16 * 10 + 10,
            queue_data.queue.get(9)
        )
        .await,
        page_number = queue_data.page_number + 1,
        total_page_number = queue_data.total_page_num + 1
    );

    fontdb.load_fonts_dir("wh_webserver/font");
    fontdb.load_system_fonts();

    let svg = usvg::Tree::from_str(
        queue.as_str(),
        &usvg::Options {
            fontdb,
            ..Default::default()
        }
        .to_ref(),
    )
    .map_err(handle_error)?;

    let mut pixmap = tiny_skia::Pixmap::new(800, 855).unwrap();

    resvg::render(&svg, usvg::FitTo::Original, pixmap.as_mut());
    //Ok((ContentType::SVG, leaderboard.into_bytes()))
    Ok((ContentType::PNG, pixmap.encode_png().map_err(handle_error)?))
}

#[get("/rank/<guildid>/<userid>")]
async fn get_rank(
    cachehttp: &State<CacheHttp>,
    data: &State<Data>,
    guildid: u64,
    userid: u64,
) -> Result<(ContentType, Vec<u8>), (Status, String)> {
    let lock = data.read().await;
    let db = lock.get::<wh_database::shared::DatabaseKey>().unwrap();
    let rules = wh_config::shared::read_config_or_default::<wh_config::shared::AllowCustomImage>(
        db, guildid,
    )
    .await;

    let rules = match rules {
        Err(e) => return Err(handle_error(&*e)),
        Ok(v) => v,
    };

    let roles = query!(
        "
        SELECT * FROM role_points 
        WHERE guildid = $1::int8
        ORDER BY points DESC
        ",
        wh_database::shared::Id(guildid) as _
    )
    .fetch_all(db)
    .await
    .map_err(handle_error)?;
    let user_points = query!(
        "SELECT points FROM user_points WHERE userid = $1::int8 AND guildid = $2::int8",
        wh_database::shared::Id(userid) as _,
        wh_database::shared::Id(guildid) as _
    )
    .fetch_optional(db)
    .await
    .map_err(handle_error)?
    .map(|r| r.points)
    .unwrap_or(0);

    let iter = roles
        .iter()
        .enumerate()
        .filter(|(_, r)| r.points <= user_points);
    let under = iter.collect::<Vec<_>>();
    // dbg!(under);
    let above = if let Some((index, _)) = under.first() {
        Some(if *index > 0 { *index - 1 } else { *index })
    } else {
        None
    };
    let mut percent = 100;
    let mut next = 0;
    let current = user_points;
    let mut width = 400;

    if let Some(index) = above {
        let above = roles.get(index).unwrap();
        next = above.points;
        if next != 0 {
            percent = ((current as f32 / next as f32) * 100.0).clamp(0.0, 100.0) as i32;

            debug!("next != 0: {}/{} => {}%", current, next, percent);
        } else {
            percent = 100;
        }
        width = (400f32 * (percent as f32 / 100.0)).clamp(0.0, 400.0) as i32 + 1;
    }
    let user = UserId(userid)
        .to_user(&cachehttp.http)
        .await
        .map_err(handle_error)?;
    let req = reqwest::get({
        let mut url = user.face();
        debug!("face: {}", url);
        let idx = url.find("webp");
        match idx {
            Some(idx) => url.replace_range(idx.., "png?size=256"),
            None => {
                let idx = url.find("gif").unwrap();
                url.replace_range(idx.., "png?size=128")
            }
        }

        url
    })
    .await
    .map_err(handle_error)?;

    let url = format!(
        "data:image/png;base64,{}",
        base64::encode(&req.bytes().await.map_err(handle_error)?)
    );

    let data = format!(
        include_str!("imgs/rank.svg"),
        percent = percent,
        next = next,
        current = current,
        width = width,
        username = user.tag(),
        img = url,
        background_image = {
            let path_str = format!(
                "{base}/images/rank/{guild}_{user}.png",
                base = std::env::var("WH_BASE_FS").expect("You need to provide WH_BASE_FS"),
                guild = guildid,
                user = userid
            );
            let path = &std::path::Path::new(path_str.as_str());
            let mut data = String::from("data:image/png;base64,");

            if path.exists()
                && (rules.default || rules.whitelist.contains(&userid))
                && !rules.blacklist.contains(&userid)
            {
                let img_data = rocket::tokio::spawn(async move { std::fs::read(&path_str) })
                    .await
                    .unwrap()
                    .map_err(handle_error)?;
                base64::encode_config_buf(&img_data, base64::STANDARD, &mut data);
            } else {
                let img_data = include_bytes!("imgs/blank_rank.png");
                base64::encode_config_buf(&img_data, base64::STANDARD, &mut data);
            }
            data
        }
    );

    let mut fontdb = usvg::fontdb::Database::new();

    fontdb.load_fonts_dir("wh_webserver/font");

    // return Ok((ContentType::SVG, data.into_bytes()));
    let pixmap = rocket::tokio::spawn(async move {
        let svg = usvg::Tree::from_str(
            data.as_str(),
            &usvg::Options {
                fontdb,
                ..Default::default()
            }
            .to_ref(),
        )
        .expect("Error when creating tree");
        let mut pixmap = tiny_skia::Pixmap::new(500, 200).unwrap();
        resvg::render(&svg, usvg::FitTo::Original, pixmap.as_mut());
        pixmap
    })
    .await
    .unwrap();
    Ok((ContentType::PNG, pixmap.encode_png().map_err(handle_error)?))
}

pub struct Discord;

#[get("/login")]
fn discord_login(
    oauth2: rocket_oauth2::OAuth2<Discord>,
    cookies: &rocket::http::CookieJar<'_>,
) -> rocket::response::Redirect {
    oauth2
        .get_redirect(cookies, &["identify", "guilds"])
        .unwrap()
}

#[get("/auth")]
fn discord_callback(
    token: rocket_oauth2::TokenResponse<Discord>,
    cookies: &rocket::http::CookieJar<'_>,
) -> rocket::response::Redirect {
    cookies.add_private(
        rocket::http::Cookie::build("token", token.access_token().to_string())
            .same_site(rocket::http::SameSite::Lax)
            .finish(),
    );

    rocket::response::Redirect::to("/app")
}

pub fn routes() -> Vec<Route> {
    routes![
        get_leaderbord,
        get_rank,
        get_queue,
        get_now_playing,
        discord_callback,
        discord_login,
    ]
}
