extern crate dotenv;
extern crate sqlx;
extern crate tokio;
extern crate wh_config;
#[macro_use]
extern crate serde;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
struct Conf {
    list: Vec<u8>,
    s: String,
    num: u64,
    new: String,
}

impl wh_config::shared::Config for Conf {
    const KEY: &'static str = "simple.json";
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().unwrap();
    let pool = sqlx::PgPool::connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();

    let mut guard = wh_config::shared::get_config_or_default::<Conf>(&pool, 1)
        .await
        .unwrap();

    dbg!(&guard);

    guard.list.push(1);
    guard.list.push(2);
    guard.list.push(3);
    guard.list.push(4);

    let date = {
        let process = std::process::Command::new("date").output().unwrap();

        String::from_utf8(process.stdout).unwrap()
    };

    guard.s = date;

    guard.num = 7;

    wh_config::shared::set_config(guard).await.unwrap();
}
