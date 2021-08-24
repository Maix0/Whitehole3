use rocket::{
    http::{ContentType, Status},
    Route, State,
};
use serenity::model::id::{GuildId, RoleId, UserId};

use crate::{CacheHttp, Data};
fn to_string<T: std::error::Error>(x: T) -> (Status, String) {
    (Status::InternalServerError, x.to_string())
}

#[get("/leaderboard/<guildid>?<page>")]
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
    .map_err(to_string)?;
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
    .map_err(to_string)?;

    let mut role_pairs = Vec::<(u64, i64, u32)>::with_capacity(res.len());

    let roles = GuildId(guildid)
        .roles(&cachehttp.http)
        .await
        .map_err(to_string)?;
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
        include_str!("svg/leaderboard.svg"),
        header = format!(
            include_str!("svg/header.svg"),
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
    .map_err(to_string)?;

    let mut pixmap = tiny_skia::Pixmap::new(800, 600).unwrap();

    resvg::render(&svg, usvg::FitTo::Original, pixmap.as_mut());
    //Ok((ContentType::SVG, leaderboard.into_bytes()))
    Ok((ContentType::PNG, pixmap.encode_png().map_err(to_string)?))
}
async fn generate_rank_item<'a>(
    page: u16,
    iter: &mut impl Iterator<Item = (usize, &'a (u64, i64))>,
    http: &CacheHttp,
    roles: &[(u64, i64, u32)],
) -> String {
    if let Some((sub_rank, pair)) = iter.next() {
        format!(
            include_str!("svg/rankitem.svg"),
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

#[get("/rank/<guildid>/<userid>")]
async fn get_rank(
    cachehttp: &State<CacheHttp>,
    data: &State<Data>,
    guildid: u64,
    userid: u64,
) -> Result<(ContentType, Vec<u8>), (Status, String)> {
    let lock = data.read().await;
    let db = lock.get::<wh_database::shared::DatabaseKey>().unwrap();

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
    .map_err(to_string)?;
    let user_points = query!(
        "SELECT points FROM user_points WHERE userid = $1::int8 AND guildid = $2::int8",
        wh_database::shared::Id(userid) as _,
        wh_database::shared::Id(guildid) as _
    )
    .fetch_optional(db)
    .await
    .map_err(to_string)?
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
            percent = ((current / next) * 100).clamp(0, 100);
            debug!("next != 0: {}/{} => {}%", current, next, percent);
        } else {
            percent = 100;
        }
        width = (400f32 * (percent as f32 / 100.0)).clamp(0.0, 400.0) as i32 + 1;
    }
    let user = UserId(userid)
        .to_user(&cachehttp.http)
        .await
        .map_err(to_string)?;
    let req = reqwest::get({
        let mut url = user.face();
        debug!("face: {}", url);
        let idx = url.find("webp");
        match idx {
            Some(idx) => url.replace_range(idx.., "png?size=256"),
            None => {
                let idx = url.find("gif").unwrap();
                url.replace_range(idx.., "png?size=256")
            }
        }

        url
    })
    .await
    .map_err(to_string)?;

    let url = format!(
        "data:image/png;base64,{}",
        base64::encode(&req.bytes().await.map_err(to_string)?)
    );

    let data = format!(
        include_str!("svg/rank.svg"),
        percent = percent,
        next = next,
        current = current,
        width = width,
        username = user.tag(),
        img = url
    );

    let mut fontdb = usvg::fontdb::Database::new();

    fontdb
        .load_font_file("wh_webserver/font/UbuntuMono-R.ttf")
        .map_err(to_string)?;

    let svg = usvg::Tree::from_str(
        data.as_str(),
        &usvg::Options {
            fontdb,
            ..Default::default()
        }
        .to_ref(),
    )
    .map_err(to_string)?;

    let mut pixmap = tiny_skia::Pixmap::new(500, 200).unwrap();

    resvg::render(&svg, usvg::FitTo::Original, pixmap.as_mut());
    //Ok((ContentType::SVG, data.into_bytes()))
    Ok((ContentType::PNG, pixmap.encode_png().map_err(to_string)?))
}

pub fn routes() -> Vec<Route> {
    routes![get_leaderbord, get_rank]
}
